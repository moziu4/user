use mongodb::bson::oid::ObjectId;
use perms::Role;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationShipData
{
    pub _id:         Option<ObjectId>,
    pub roles:       Role,
    pub permissions: Vec<u32>,
}