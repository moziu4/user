use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewUser
{
    pub username: String,
    pub email:    String,
    pub password: String,
    pub name:     String,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String,
}