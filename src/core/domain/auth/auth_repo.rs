use async_trait::async_trait;


#[async_trait]
pub trait AuthRepo
{
  // async fn do_login (&self, user_auth: AuthLogin) -> Result<String, AuthError>;
}
