use crate::core::domain::auth::AuthEntity;
use crate::data::access::auth_repo::MongoAuthRepo;
use crate::data::catalog_importer::MongoCatalogRepo;
use crate::error::{ServiceError, ServiceResult};


pub struct CatalogsOps<'a>
{
    repo: &'a MongoCatalogRepo,
    auth_repo: &'a MongoAuthRepo,

}
impl <'a>CatalogsOps<'a>
{
    pub fn new(repo: &'a MongoCatalogRepo, auth_repo: &'a MongoAuthRepo
    ) -> Self
    {
        Self {repo, auth_repo }
    }
    
    pub async fn sync_catalogs(&self) -> ServiceResult<()>
    {
        let importer = self.repo;
        
        importer.import_perm_relationships().await?;
        self.update_perms_in_users().await?;
        println!("Import Catalog Permissions");
        Ok(())
    }
    
    async fn update_perms_in_users(&self) -> ServiceResult<()>
    {
        let relationships = self.repo.fetch_perm_relationships().await?;
        let auths = self.auth_repo.fetch_all().await.map_err(|_| ServiceError::FetchUserError)?;
        for auth in auths
        {
            let mut auth_entity = AuthEntity::new(auth.clone(), self.auth_repo).await;
            

            if let Some(perms) = relationships.get(&auth.roles) {
                if auth.permissions != *perms {
                    auth_entity.update_id(auth._id.unwrap()).await;
                    auth_entity.update_permissions(perms.clone()).await;
                    auth_entity.save().await.map_err(|_| {
                        ServiceError::UpdateUserError
                    })?;
                }
            } else {
                if !auth.permissions.is_empty() {
                    auth_entity.update_permissions(Vec::new()).await;
                    auth_entity.save().await.map_err(|_| {
                        ServiceError::UpdateUserError
                    })?;
                }
            }
        }
        
        Ok(())
    }


}