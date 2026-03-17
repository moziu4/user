use futures_util::TryFutureExt;
use crate::core::domain::auth::auth_type::Role;
use crate::utils::domains_ids::{TenantID, AgencyID};
use perms::{AuthID, UserID};
use serde::{Deserialize, Serialize};
use crate::core::domain::auth::auth_error::AuthError;
use crate::data::access::auth_repo::MongoAuthRepo;
use crate::data::access::user_repo::MongoUserRepo;


pub mod auth_repo;

pub mod auth_type;
pub mod auth_error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth
{
    pub _id:         Option<AuthID>,
    pub user_id:     UserID,
    pub username:    String,
    pub email:       String,
    pub password:    String,
    pub role_id:     u32,
    pub permissions: Vec<u32>,
}

#[derive(Clone)]
pub struct AuthEntity<'a>
{
    props: Auth,
    repo: &'a MongoAuthRepo,
}

impl<'a>AuthEntity<'a>
{
    pub async fn new(new_auth: Auth, repo: &'a MongoAuthRepo) -> Self
    {
        Self { repo,
        props: Auth{
            _id: new_auth._id,
            user_id: new_auth.user_id,
            username: new_auth.username,
            email: new_auth.email,
            password: new_auth.password,
            role_id: new_auth.role_id,
            permissions: new_auth.permissions,
        }}
    }
    
    pub async fn update_id(& mut self, id: AuthID )
    {
        self.props._id = Some(id);
    }
    
    pub async fn create(self) -> Result<Auth, AuthError>
    {
        self.repo.create(self.props).await
    }
    
    pub async fn update_role(&mut self, role_id: u32)
    {
        self.props.role_id = role_id;
    }
        
    pub async fn update_permissions(&mut self, permissions: Vec<u32>)
    {
        self.props.permissions = permissions;
    }
    pub async fn save(self) -> Result<Auth, auth_error::AuthError>
    {
        println!("{:?}", self.props);
        self.repo.save(self.props).await
    }

    pub async fn login_contextual(&self, user_repo: &MongoUserRepo, tenant_id: Option<TenantID>, agency_id: Option<AgencyID>) -> Result<perms::Auth, AuthError> {
        let role = Role::from_id(self.props.role_id).unwrap_or(Role::Guest);
        
        // Si es SuperAdmin, ignoramos el contexto y devolvemos todos los permisos
        if role == Role::SuperAdmin {
            return Ok(self.props.clone().into());
        }

        // Si no es SuperAdmin, buscamos la membresía para obtener el rol contextual
        let memberships = user_repo.fetch_memberships_by_user(self.props.user_id.clone())
            .map_err(|_| AuthError::MembershipNotFound).await?;
        
        let membership = memberships.first().ok_or(AuthError::MembershipNotFound)?;

        let contextual_role_id = if let Some(t_id) = tenant_id {
            membership.tenants.iter()
                .find(|t| t.tenant_id == t_id)
                .map(|t| t.role_id)
        } else if let Some(a_id) = agency_id {
            membership.agencies.iter()
                .find(|a| a.agency_id == a_id)
                .map(|a| a.role_id)
        } else {
            None
        };

        let final_role_id = contextual_role_id.unwrap_or(self.props.role_id);
        
        // Aquí podrías recargar los permisos basados en el final_role_id si fuera necesario
        // Por ahora, asumimos que Auth ya tiene los permisos básicos o los que corresponden al rol principal
        
        Ok(perms::token::Auth {
            _id: Some(self.props._id.clone().expect("Auth should have an ID at login")),
            user_id: self.props.user_id.clone(),
            username: self.props.username.clone(),
            email: self.props.email.clone(),
            password: self.props.password.clone(),
            roles: Role::from_id(final_role_id).unwrap_or(Role::Guest),
            permissions: self.props.permissions.clone(),
        })
    }
}

impl From<Auth> for perms::token::Auth {
    fn from(service_auth: Auth) -> Self {
        perms::token::Auth {
            _id: Some(service_auth._id.expect("Auth should have an ID at conversion")),
            user_id: service_auth.user_id,
            username: service_auth.username,
            email: service_auth.email,
            password: service_auth.password,
            roles: Role::from_id(service_auth.role_id).unwrap_or(Role::Guest),
            permissions: service_auth.permissions,
        }
    }
}

