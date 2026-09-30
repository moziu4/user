use std::{fmt, str::FromStr};
pub use perms::Role;
use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{TenantID, OrganizationID};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthLogin
{
    pub username:  String,
    pub password:  String,
    pub tenant_id: Option<TenantID>,
    pub agency_id: Option<OrganizationID>,
}

pub use perms::Claims;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token
{
    pub token: String,
}
