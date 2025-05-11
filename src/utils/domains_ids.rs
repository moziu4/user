use std::str::FromStr;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserID(ObjectId);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuthID(ObjectId);

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

implement_id!(UserID);
implement_id!(AuthID);
implement_id!(PermID);

impl std::fmt::Display for UserID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for AuthID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for PermID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}
