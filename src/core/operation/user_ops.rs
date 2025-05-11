use std::sync::Arc;
use actix_web::HttpRequest;
use bcrypt::{hash, DEFAULT_COST};
use crate::{
    core::domain::{
        auth::{auth_type::Role},
        perm::{perm_repo::PermRepo},
        user::{
            user_type::{NewUser},
        },
    },
};
use crate::context::Context;
use crate::core::domain::auth::auth_type::Claims;
use crate::core::domain::auth::{Auth, AuthEntity};
use crate::core::domain::user::{User, UserEntity};
use crate::core::domain::user::user_error::UserError;
use crate::data::access::auth_repo::MongoAuthRepo;
use crate::data::access::perms_repo::MongoPermRepo;
use crate::data::access::user_repo::MongoUserRepo;
use crate::utils::domains_ids::UserID;

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
    
    pub async fn create_user(&self, new_user: NewUser, public: bool) -> Result<User, UserError>
    {
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
        
        let password = hash(new_user.password, DEFAULT_COST).map_err(|err| UserError::HashPasswordError)?;
        let role;
        if public
        {
            role = Role::SuperAdmin;
        }
        else{
            role = Role::Client;
        }
        
        
        let perms = self.perm_repo
            .charge_permissions(role.to_string(),self.context)
            .await
            .map_err(|_| UserError::PermError)?;
       
        
        let auth = Auth{
            _id: None,
            user_id,
            username: user.username.clone(),
            email: user.email.clone(),
            password ,
            roles: role,
            permissions: perms,
        };
        
        let auth_entity = AuthEntity::new(auth, self.auth_repo).await;
        auth_entity.create()
            .await
            .map_err(|_| UserError::AuthError)?;
        

        Ok(user)
    }

    // pub async fn intern_new_user(&self, user: NewUser) -> Result<User, UserError>
    // {
    //     let new_user = self.user_repo.insert_user(user.clone()).await?;
    //     let role = Role::SuperAdmin.to_string();
    //     let command = "private".to_string();
    //     let perm = self.perms_repo
    //                     .charge_permissions(role)
    //                     .await
    //                     .map_err(|perms_err| UserError { message: perms_err.message })?;
    //     self.auth_repo
    //         .insert_auth(command, user, new_user.id, perm)
    //         .await
    //         .map_err(|auth_err| UserError { message: auth_err.message })?;
    // 
    // 
    //     Ok(new_user)
    // }

    pub async fn load_users(&self, req: HttpRequest) -> Result<Vec<User>, UserError>
    {
        // if !self.has_permission(req, READ_USER).await
        // {
        //     return Err(UserError { message: "Permission denied".to_string() });
        // }

        let users = self.repo.fetch_all().await?;
        Ok(users)
    }
    
    pub async fn load_user_by_id(&self, id: UserID) -> Result<User, UserError>
    {
        let user = self.repo.fetch_by_id(id).await?;
        Ok(user)
    }

    async fn has_permission(&self, req: HttpRequest, permission: u32) -> bool
    {
        if let Some(auth_header) = req.headers().get("Authorization")
        {
            if let Ok(auth_str) = auth_header.to_str()
            {
                if auth_str.starts_with("Bearer ")
                {
                    let token = &auth_str[7..];
                    let secret = std::env::var("SECRET_KEY").unwrap().to_string();
                    let validation = jsonwebtoken::Validation::default();
                    if let Ok(token_data) =
                        jsonwebtoken::decode::<Claims>(&token,
                                                       &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
                                                       &validation)
                    {
                        let claims = token_data.claims;
                        return claims.permissions.contains(&permission);
                    }
                }
            }
        }
        false
    }

      
}
