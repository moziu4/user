use async_trait::async_trait;
use crate::context::Context;
use crate::core::domain::perm::perm_error::PermError;
use crate::core::domain::perm::perm_type::PermsRelationship;
use std::collections::HashMap;

#[async_trait]
pub trait PermRepo
{
    async fn create_perms_relationship(&self, perms_relationships: Vec<PermsRelationship>, context: &Context) -> Result<(), PermError>;
    async fn charge_permissions(&self, role_id: u32, context: &Context) -> Result<Vec<String>, PermError>;
    async fn get_perms_map(&self, context: &Context) -> Result<HashMap<String, u32>, PermError>;
}
