pub use perms::Role;
use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{TenantID, OrganizationID};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthLogin
{
    pub username:  String,
    pub password:  String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContextSelection
{
    pub organization_id: OrganizationID,
    pub tenant_id: Option<TenantID>,
}

pub use perms::Claims;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token
{
    pub token: String,
}
