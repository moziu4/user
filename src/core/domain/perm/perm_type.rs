use crate::core::domain::auth::auth_type::Role;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermsRelationship
{
    pub id:    Option<u32>,
    pub role:  Role,
    pub perms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermsRelationshipDTO
{
    pub id:    Option<u32>,
    pub role:  String,
    pub perms: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo
{
    pub id:    u32,
    pub role:  Role,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentType
{
    pub id:    u32,
    pub name:  String,
}
