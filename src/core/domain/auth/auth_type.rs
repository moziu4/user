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
    sub: String,
    exp: usize,
    pub permissions: Vec<u32>,
    role: Role,
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
    Admin,
    Client,
    Visitor,
}

impl fmt::Display for Role
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", match self
        {
            Role::SuperAdmin => "SuperAdmin",
            Role::Admin => "Admin",
            Role::Client => "Client",
            Role::Visitor => "Visitor",
        })
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
            "Admin" => Ok(Role::Admin),
            "Client" => Ok(Role::Client),
            "Visitor" => Ok(Role::Visitor),
            _ => Err(()),
        }
    }
}
