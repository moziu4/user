use crate::core::domain::auth::auth_type::Role;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewUser
{
    pub username: String,
    pub email:    String,
    pub password: String,
    pub name:     String,
    pub role:     Option<Role>,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String,
}