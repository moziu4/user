use serde::{Deserialize, Serialize};
use crate::core::domain::user::user_error::UserError;
use crate::core::domain::user::user_type::NewUser;
use crate::data::access::user_repo::MongoUserRepo;
use crate::utils::domains_ids::UserID;

pub mod user_repo;
pub mod user_type;

pub mod user_error;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User
{
    pub _id:       Option<UserID>,
    pub username: String,
    pub email:    String,
    pub name:     String,
}

#[derive(Clone)]
pub struct UserEntity<'a>
{
    props: User,
    repo: &'a MongoUserRepo,
}

impl <'a> UserEntity<'a>
{
   pub async fn new(new_user: NewUser, repo: &'a MongoUserRepo ) -> Self
   {
       Self {
           repo,
           props: User {
               _id:       None,
               username: new_user.username,
               email:    new_user.email,
               name:     new_user.name,
           }
       }
   }
    
    pub async fn create(self) -> Result< User, UserError>
    {
        self.repo.create(self.props).await
    }
}

