use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;

use crate::{
    core::domain::user::user_type::{NewUser},
};
use crate::core::domain::user::User;

#[async_trait]
pub trait UserRepo
{
    // async fn insert_user(&self, user_new: UserNew) -> Result<User, UserError>;
    // async fn load_users(&self) -> Result<Vec<User>, UserError>;
    // async fn load_user_by_username(&self, username: String) -> Result<User, UserError>;
    // async fn load_user_by_id(&self, user_id: ObjectId) -> Result<User, UserError>;
}
