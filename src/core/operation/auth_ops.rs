use std::env;
use bcrypt::verify;
use perms::Token;
use crate::{
    core::domain::auth::{auth_type::AuthLogin},
    data::access::user_repo::MongoUserRepo,
    data::access::perms_repo::MongoPermRepo,
    context::Context,
};
use crate::core::domain::auth::{Auth, AuthEntity};
use crate::core::domain::auth::auth_error::AuthError;
use crate::data::access::auth_repo::MongoAuthRepo;

pub struct AuthOps<'a>
{
    repo: &'a MongoAuthRepo,
    user_repo: &'a MongoUserRepo,
    perm_repo: &'a MongoPermRepo,
    context: &'a Context,
}

impl<'a> AuthOps<'a>
{
    pub fn new(repo: &'a MongoAuthRepo, user_repo: &'a MongoUserRepo, perm_repo: &'a MongoPermRepo, context: &'a Context) -> Self
    {
        Self {repo, user_repo, perm_repo, context}
    }
    
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
        let auth = self.repo.fetch_by_username(auth_login.clone().username).await?;

        let is_password_valid = verify(auth_login.password.clone(), &auth.password)
            .map_err(|_| AuthError::IncorrectPassword)?;

        if !is_password_valid {
            return Err(AuthError::IncorrectPassword);
        }
        
        let auth_entity = AuthEntity::new(auth.clone(), self.repo).await;
        let auth_perms = auth_entity.login_contextual(self.user_repo, self.perm_repo, auth_login.tenant_id, auth_login.agency_id, self.context).await?;

        let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
        let token = Token::new(secret, auth_perms).map_err(|_| AuthError::PermLibError)?;
        Ok(token)
    }
}
