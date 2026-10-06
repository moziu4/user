use crate::core::domain::auth::auth_type::Role;
use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{PhoneID, AddressID, TenantID, OrganizationID};
use perms::UserID;
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewUser
{
    pub username: String,
    pub email:    String,
    pub password: String,
    pub name:     String,
    pub surname_1: Option<String>,
    pub surname_2: Option<String>,
    pub membership: Option<Membership>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Membership {
    pub role_id:   Option<u32>,
    pub organization_id: Option<OrganizationID>,
    pub tenant_id: Option<TenantID>,
    pub status:    MembershipStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MembershipStatus {
    Active,
    Inactive,
    Pending,
    Unverified,
    Suspended,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IdentityDocumentType {
    DNI,
    NIE,
    Passport,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentityDocument {
    pub _id: Option<ObjectId>,
    pub doc_type: IdentityDocumentType,
    pub number: String,
    pub country: String,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
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