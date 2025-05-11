use crate::{
    core::domain::auth::{auth_type::AuthLogin},
};
use crate::core::domain::auth::{Auth, AuthEntity};
use crate::core::domain::auth::auth_error::AuthError;
use crate::core::domain::auth::auth_type::Token;
use crate::data::access::auth_repo::MongoAuthRepo;

pub struct AuthOps<'a>
{
    repo: &'a MongoAuthRepo,
}

impl<'a> AuthOps<'a>
{
    pub fn new(repo: &'a MongoAuthRepo) -> Self  {Self {repo}}
    
    pub async fn create_auth(&self, auth: Auth) -> Result<Auth, AuthError>
    {
        if self.repo.fetch_by_username(auth.clone().username).await.is_ok() {
            return Err(AuthError::AlreadyUsernameExists)
        }
        if self.repo.fetch_by_email(auth.clone().email).await.is_ok() {
            return Err(AuthError::AlreadyEmailExists)       
        }
        let auth_entity = AuthEntity::new(auth.clone(), self.repo).await;
        let auth = auth_entity.create().await?;
        Ok(auth)
    }
    
    pub async fn do_login (&self, auth_login: AuthLogin ) -> Result<Token, AuthError>
    {
        let auth = self.repo.fetch_by_username(auth_login.username).await?;
        if auth.password != auth_login.password {
           return  Err(AuthError::IncorrectPassword)
        };
        Token::new(auth)
    }
}
