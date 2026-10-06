use perms::UserID;
use serde::{Deserialize, Serialize};
use crate::core::domain::user::user_error::UserError;
use crate::core::domain::user::user_type::{NewUser, IdentityDocument};
use crate::data::access::user_repo::MongoUserRepo;
use chrono::{DateTime, Utc};


pub mod user_repo;
pub mod user_type;

pub mod user_error;



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Unverified,
    Suspended,
    Deleted,
    Anonymized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User
{
    pub _id:           Option<UserID>,
    pub username:      String,
    pub email:         String,
    pub name:          String,
    pub surname_1:     Option<String>,
    pub surname_2:     Option<String>,
    pub documents:     Vec<IdentityDocument>,
    pub status:        Status,
    pub deleted_at:    Option<DateTime<Utc>>,
    pub anonymized_at: Option<DateTime<Utc>>,
    pub gpd_erased:    bool,
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
       Self::new_with_status(new_user, Status::Active, repo).await
   }

   pub async fn new_with_status(new_user: NewUser, status: Status, repo: &'a MongoUserRepo) -> Self
   {
       Self {
           repo,
           props: User {
               _id:           None,
               username:      new_user.username,
               email:         new_user.email,
               name:          new_user.name,
               surname_1:     new_user.surname_1,
               surname_2:     new_user.surname_2,
               documents:     Vec::new(),
               status,
               deleted_at:    None,
               anonymized_at: None,
               gpd_erased:    false,
           }
       }
   }
    
    pub async fn create(self) -> Result< User, UserError>
    {
        self.repo.create(self.props).await
    }
}

