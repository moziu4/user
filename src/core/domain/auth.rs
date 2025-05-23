use serde::{Deserialize, Serialize};
use crate::core::domain::auth::auth_error::AuthError;
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
    
    pub async fn update_id(& mut self, id: AuthID )
    {
        self.props._id = Some(id);
    }
    
    pub async fn create(self) -> Result<Auth, AuthError>
    {
        self.repo.create(self.props).await
    }
    
    pub async fn update_role(&mut self, role: Role)
    {
        self.props.roles = role;
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
}

impl From<Auth> for perms::token::Auth {
    fn from(service_auth: Auth) -> Self {
        perms::token::Auth {
            _id: Option::from(service_auth._id.expect("REASON").to_string()),
            user_id: service_auth.user_id.to_string(),
            username: service_auth.username,
            email: service_auth.email,
            password: service_auth.password,
            roles: service_auth.roles.to_string(),
            permissions: service_auth.permissions,
        }
    }
}

