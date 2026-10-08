use std::str::FromStr;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PermID(ObjectId);

macro_rules! implement_id {
    ($type:ident) => {
        impl $type
        {
            pub fn new() -> Self
            {
                Self(ObjectId::new())
            }

            pub fn from_object_id(id: ObjectId) -> Self
            {
                Self(id)
            }

            pub fn value(&self) -> ObjectId
            {
                self.0
            }

            pub fn parse_str(s: &str) -> Result<Self, mongodb::bson::oid::Error>
            {
                Ok(Self(ObjectId::from_str(s)?))
            }
        }

        impl From<ObjectId> for $type
        {
            fn from(id: ObjectId) -> Self
            {
                Self(id)
            }
        }

        impl From<$type> for ObjectId
        {
            fn from(id: $type) -> ObjectId
            {
                id.0
            }
        }
    };
}


implement_id!(PermID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct OrganizationID(ObjectId);
implement_id!(OrganizationID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TenantID(ObjectId);
implement_id!(TenantID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MembershipID(ObjectId);
implement_id!(MembershipID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PhoneID(ObjectId);
implement_id!(PhoneID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AddressID(ObjectId);
implement_id!(AddressID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FactID(ObjectId);
implement_id!(FactID);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlanID(ObjectId);
implement_id!(PlanID);

impl std::fmt::Display for PermID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

macro_rules! implement_display {
    ($type:ident) => {
        impl std::fmt::Display for $type
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
            {
                write!(f, "{}", self.0)
            }
        }
    };
}

implement_display!(OrganizationID);
implement_display!(TenantID);
implement_display!(MembershipID);
implement_display!(PhoneID);
implement_display!(AddressID);
implement_display!(FactID);
