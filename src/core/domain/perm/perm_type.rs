use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::core::domain::auth::auth_type::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermsRelationship
{
    pub id:    Option<ObjectId>,
    pub role:  Role,
    pub perms: Vec<u32>,
}
