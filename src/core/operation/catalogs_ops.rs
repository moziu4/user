use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::{DocumentType, RoleInfo};
use mongodb::bson::{doc, Document};
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
        
        importer.import_perms().await?;
        importer.import_perm_relationships().await?;
        importer.import_document_types().await?;
        self.update_perms_in_users().await?;
        println!("Import Catalog Permissions, Relationships and Document Types");
        Ok(())
    }
    
    pub async fn get_roles(&self) -> ServiceResult<Vec<RoleInfo>> {
        self.repo.fetch_roles().await
    }

    pub async fn get_document_types(&self) -> ServiceResult<Vec<DocumentType>> {
        self.repo.fetch_document_types().await
    }
    
    async fn update_perms_in_users(&self) -> ServiceResult<()>
    {
        let relationships = self.repo.fetch_perm_relationships().await?;
        
        // Optimización: Usar update_many para cada rol para actualizar todos los usuarios de ese rol de una vez.
        // Esto es mucho más eficiente que iterar sobre miles de usuarios.
        for (role, perms) in relationships {
            let filter = doc! { 
                "role_id": role.to_id(),
                "permissions": { "$ne": perms.clone() } // Solo actualizar si los permisos son diferentes
            };
            let update = doc! { "$set": { "permissions": perms } };
            
            let coll = self.auth_repo.get_collection();
            
            match coll.update_many(filter, update).await {
                Ok(result) => {
                    if result.modified_count > 0 {
                        println!("Actualizados {} usuarios para el rol {:?}", result.modified_count, role);
                    }
                }
                Err(e) => {
                    eprintln!("Error al actualizar usuarios para el rol {:?}: {}", role, e);
                    return Err(ServiceError::UpdateUserError);
                }
            }
        }
        
        Ok(())
    }


}