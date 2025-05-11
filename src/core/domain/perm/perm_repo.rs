use async_trait::async_trait;
use crate::context::Context;
use crate::core::domain::perm::perm_error::PermError;
use crate::core::domain::perm::perm_type::PermsRelationship;

#[async_trait]
pub trait PermRepo
{
    async fn create_perms_relationship(&self, perms_relationships: Vec<PermsRelationship>, context: &Context) -> Result<(), PermError>;
    async fn charge_permissions(&self, command: String, context: &Context) -> Result<Vec<u32>, PermError>;
}
