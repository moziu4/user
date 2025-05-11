use std::{env, fmt, str::FromStr};
use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use crate::core::domain::auth::Auth;
use crate::core::domain::auth::auth_error::AuthError;

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

impl Token
{
    pub fn new(auth: Auth) -> Result<Token, AuthError>
    {
        let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
            let exp = (since_the_epoch.as_secs() + 3600) as usize;
        
            let claims = Claims {
                sub: auth.username,
                role: auth.roles,
                exp,
                permissions: auth.permissions,
            };
        
            let secret = env::var("SECRET_KEY").unwrap().to_string();
            let token =
                encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).map_err(|_| AuthError::FailToCreateToken)?;
        
            Ok(Token{token})
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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
