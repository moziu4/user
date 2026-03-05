use crate::core::domain::auth::auth_type::Role;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermsRelationship
{
    pub role:  Role,
    pub perms: Vec<u32>,
}
