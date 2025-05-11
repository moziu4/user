use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;

use crate::core::domain::{
    auth::auth_type::AuthLogin,
    user::user_type::NewUser,
};
use crate::core::domain::auth::Auth;
use crate::core::domain::auth::auth_error::AuthError;

#[async_trait]
pub trait AuthRepo
{
  // async fn do_login (&self, user_auth: AuthLogin) -> Result<String, AuthError>;
}
