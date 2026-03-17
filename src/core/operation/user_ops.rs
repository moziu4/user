use std::env;
use actix_web::HttpRequest;
use bcrypt::{hash};
use perms::{has_permission, UserID};
use crate::{
    core::domain::{
        auth::auth_type::Role,
        perm::{perm_repo::PermRepo},
        user::{
            user_type::{NewUser},
        },
        membership::Membership,
    },
};
use crate::context::Context;
use crate::core::domain::auth::{Auth, AuthEntity};
use crate::core::domain::perm::perm_cat::{CREATE_USER, CREATE_USERS_AGENCIES, CREATE_USERS_TENANTS, READ_USERS};
use crate::core::domain::user::{User, UserEntity};
use crate::core::domain::user::user_error::UserError;
use crate::data::access::auth_repo::MongoAuthRepo;
use crate::data::access::perms_repo::MongoPermRepo;
use crate::data::access::user_repo::MongoUserRepo;


pub struct UserOps<'a>
{
    repo:  &'a MongoUserRepo,
    perm_repo: &'a dyn PermRepo,
    auth_repo: &'a MongoAuthRepo,
    context: &'a Context,
}


impl<'a> UserOps<'a>
{
    pub async fn new(repo: &'a MongoUserRepo, perm_repo: &'a MongoPermRepo, auth_repo: &'a MongoAuthRepo, context: &'a Context) -> Self
    {
        Self { repo,
        perm_repo,
        auth_repo,
            context
        }
    }

    fn can_create_role(requester_role_id: u32, target_role_id: u32) -> bool {
        let requester_role = Role::from_id(requester_role_id).unwrap_or(Role::Guest);
        let target_role = Role::from_id(target_role_id).unwrap_or(Role::Guest);

        // Un usuario NO puede crear a otro con su mismo rol o superior.
        // La jerarquía en `Role` es (de mayor a menor):
        // SuperAdmin, AgencyOwner, AgencyAdmin, AgencyMember, TenantAdmin, Editor, Client, Guest.

        // En Rust, si el enum deriva `PartialOrd`, el orden es el de declaración.
        // Role ya deriva `PartialOrd`, por lo que podemos comparar directamente.
        // SuperAdmin (0) < AgencyOwner (1) < ... < Guest (7)
        // Por tanto, si queremos que el CREADOR tenga MAYOR rango, su valor numérico debe ser MENOR.
        
        requester_role < target_role
    }

    pub async fn create_user(&self, new_user: NewUser, req: HttpRequest) -> Result<User, UserError>
    {
        let _secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");

        // 1. Determine target role
        let target_role_id = new_user.role_id.unwrap_or(Role::Client as u32);
        let target_role = Role::from_id(target_role_id).unwrap_or(Role::Client);

        // 2. Extract requester info (permissions and role)
        // Manual extraction for now as we don't have the function in perms
        let _auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(UserError::Unauthorized)?;

        // This is a placeholder. In a real scenario, you'd verify the JWT here.
        // For now, we'll assume the token is valid and deserialize it.
        // Alternatively, use perms::decode if available.
        let requester_claims: crate::core::domain::auth::auth_type::Claims = serde_json::from_str("{}") // Placeholder
             .map_err(|_| UserError::Unauthorized)?;

        let requester_perms = requester_claims.permissions;
        let requester_role_id = requester_claims.role_id;
        let requester_role = Role::from_id(requester_role_id).unwrap_or(Role::Guest);

        // 3. Check basic permission to create users based on target role
        let has_basic_create = requester_perms.contains(&CREATE_USER);
        let has_agency_create = requester_perms.contains(&CREATE_USERS_AGENCIES);
        let has_tenant_create = requester_perms.contains(&CREATE_USERS_TENANTS);

        let is_target_agency = matches!(target_role, Role::AgencyOwner | Role::AgencyAdmin | Role::AgencyMember);
        let is_target_tenant = matches!(target_role, Role::TenantAdmin | Role::Editor | Role::Client | Role::Guest);

        // Validación contextual:
        // - Los de Agencia se crean en la "pantalla de agencias" (requiere CREATE_USERS_AGENCIES)
        // - Los de Tenant se crean en la "pantalla de tenant" (requiere CREATE_USERS_TENANTS)
        let can_proceed = if has_basic_create {
            true // SuperAdmin con permiso global puede todo (sujeto a jerarquía)
        } else if is_target_agency {
            has_agency_create
        } else if is_target_tenant {
            has_tenant_create
        } else {
            false
        };

        if !can_proceed {
            return Err(UserError::NotHasPermission);
        }

        // 4. Enforce strict role hierarchy
        if !Self::can_create_role(requester_role_id, target_role_id) {
            return Err(UserError::InsufficientPrivileges);
        }

        // 5. Basic user validation
        if self.repo.fetch_by_email(new_user.email.clone()).await.is_ok()
        {
            return Err(UserError::EmailIsUsed)
        }
        if !new_user.email.contains('@')
        {
            return Err(UserError::IncorrectFormatEmail)
        }

        let user_entity = UserEntity::new(new_user.clone(), self.repo).await;
        let user = user_entity.create().await?;
        let user_id = match user.clone()._id
        {
            Some(id) => id,
            None => return Err(UserError::InvalidUserId)
        };

        let password = hash(new_user.password, 10).map_err(|_err| UserError::HashPasswordError)?;

        // 6. Charge permissions for the target role
        let perms = self.perm_repo
            .charge_permissions(target_role_id, self.context)
            .await
            .map_err(|_| UserError::PermError)?;

        let auth = Auth{
            _id: None,
            user_id: user_id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            password ,
            role_id: target_role_id,
            permissions: perms,
        };

        let auth_entity = AuthEntity::new(auth, self.auth_repo).await;
        auth_entity.create()
            .await
            .map_err(|_| UserError::AuthError)?;

        // 7. Create Membership if agency_id or tenant_id is provided
        if new_user.agency_id.is_some() || new_user.tenant_id.is_some() {
            let mut membership = Membership::new(user_id.clone());
            if let Some(agency_id) = new_user.agency_id {
                membership.add_agency(agency_id, target_role_id);
            }
            if let Some(tenant_id) = new_user.tenant_id {
                membership.add_tenant(tenant_id, target_role_id);
            }
            self.repo.create_membership(membership).await?;
        }

        // 8. Publish NATS event
        if let Some(nats_service) = &self.context.nats_service {
            nats_service.publish_user_event(
                user_id.to_string(),
                user.username.clone(),
                user.email.clone(),
                "created"
            ).await;
        }

        Ok(user)
    }

    pub async fn load_users(&self, req: HttpRequest) -> Result<Vec<User>, UserError>
    {
        let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
        if !has_permission(secret, req, READ_USERS).await
        {
            return Err(UserError::NotHasPermission);
        }

        let users = self.repo.fetch_all().await?;
        Ok(users)
    }
    
    pub async fn load_user_by_id(&self, id: UserID) -> Result<User, UserError>
    {
        let user = self.repo.fetch_by_id(id).await?;
        Ok(user)
    }

    pub async fn change_password(&self, user_id: UserID, new_password: String) -> Result<(), UserError>
    {
        let mut auth = self.auth_repo.fetch_by_user_id(user_id).await.map_err(|_| UserError::AuthError)?;
        let hashed_password = hash(new_password, 10).map_err(|_| UserError::HashPasswordError)?;
        auth.password = hashed_password;
        
        self.auth_repo.save(auth.clone()).await.map_err(|_| UserError::AuthError)?;

        // Publish NATS event
        if let Some(nats_service) = &self.context.nats_service {
            nats_service.publish_user_event(
                auth.user_id.to_string(),
                auth.username.clone(),
                auth.email.clone(),
                "password_changed"
            ).await;
        }

        Ok(())
    }
          
}
