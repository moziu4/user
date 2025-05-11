use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use crate::core::domain::auth::auth_type::Role;
use crate::data::access::auth_repo::MongoAuthRepo;
use crate::utils::domains_ids::{AuthID, UserID};

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
    pub roles:       Role,
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
            _id: None,
            user_id: new_auth.user_id,
            username: new_auth.username,
            email: new_auth.email,
            password: new_auth.password,
            roles: new_auth.roles,
            permissions: new_auth.permissions,
        }}
    }
    
    pub async fn create(self) -> Result<Auth, auth_error::AuthError>
    {
        self.repo.create(self.props).await
    }
}