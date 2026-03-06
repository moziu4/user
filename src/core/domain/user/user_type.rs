use crate::core::domain::auth::auth_type::Role;
use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{PhoneID, AddressID, TenantID};
use perms::UserID;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PhoneStatus {
    Active,
    Inactive,
    Unverified,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Phone {
    pub _id:       Option<PhoneID>,
    pub status:    PhoneStatus,
    pub user_id:   UserID,
    pub tenant_id: Option<TenantID>,
    pub primary:   bool,
    pub number:    String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AddressStatus {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Address {
    pub _id:        Option<AddressID>,
    pub status:     AddressStatus,
    pub user_id:    UserID,
    pub tenant_id:  Option<TenantID>,
    pub primary:    bool,
    pub street:     String,
    pub city:       String,
    pub country:    String,
    pub postal_code: String,
}