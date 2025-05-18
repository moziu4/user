use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::core::domain::auth::auth_type::Role;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationShipData
{
    pub _id:         Option<ObjectId>,
    pub roles:       Role,
    pub permissions: Vec<u32>,
}