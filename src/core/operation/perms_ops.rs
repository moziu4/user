
use crate::{core::domain::perm::{
    perm_type::{PermsRelationship},
}};
use crate::context::Context;
use crate::core::domain::perm::{Perm, PermEntity};
use crate::core::domain::perm::perm_error::PermError;
use crate::core::domain::perm::perm_repo::PermRepo;
use crate::data::access::perms_repo::MongoPermRepo;

pub struct PermOps<'a>
{
    repo: &'a MongoPermRepo,
    context : &'a Context,
    
}

impl<'a> PermOps<'a>
{
    pub fn new(repo: &'a MongoPermRepo, context:&'a Context) -> Self{
        Self{
            repo,
            context
        }
      
    }
   
    pub async fn create_perms(&self, perm: Perm) -> Result<Perm, PermError>
    {
        let perm_entity = PermEntity::new(perm, self.repo).await;
        let perm = perm_entity.create().await?;
        Ok(perm)
    }

    pub async fn create_perms_relationship(&self, perms_relationships: Vec<PermsRelationship>)
                                           -> Result<(), PermError>
    {
        self.repo
            .create_perms_relationship(perms_relationships, self.context)
            .await
            .map_err(|_auth_err| PermError::PermRelationShipNotCreated)?;
        Ok(())
    }
}
