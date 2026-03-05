use std::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthLogin
{
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub permissions: Vec<u32>,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token
{
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Hash, Eq)]
pub enum Role
{
    SuperAdmin,
    AgencyOwner,
    AgencyAdmin,
    AgencyMember,
    TenantAdmin,
    Editor,
    Client,
    Guest,
}

impl From<Role> for perms::Role {
    fn from(role: Role) -> Self {
        match role {
            Role::SuperAdmin => perms::Role::SuperAdmin,
            Role::Client => perms::Role::Client,
            // Map others to Client or something sensible if they don't exist in perms
            _ => perms::Role::Client,
        }
    }
}

impl fmt::Display for Role
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Role
{
    type Err = ();

    fn from_str(input: &str) -> Result<Role, Self::Err>
    {
        match input
        {
            "SuperAdmin" => Ok(Role::SuperAdmin),
            "AgencyOwner" => Ok(Role::AgencyOwner),
            "AgencyAdmin" => Ok(Role::AgencyAdmin),
            "AgencyMember" => Ok(Role::AgencyMember),
            "TenantAdmin" => Ok(Role::TenantAdmin),
            "Editor" => Ok(Role::Editor),
            "Client" => Ok(Role::Client),
            "Guest" => Ok(Role::Guest),
            _ => Err(()),
        }
    }
}
