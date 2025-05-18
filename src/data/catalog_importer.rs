use std::collections::HashMap;
use std::{env, fs};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, from_document, to_bson, Document};
use mongodb::{Client, Collection};
use crate::core::domain::auth::Auth;
use crate::core::domain::auth::auth_error::AuthError;
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::PermsRelationship;
use crate::error::{ServiceError, ServiceResult};
use crate::handlers::http::catalogs::catalog_data::RelationShipData;

#[derive(Clone)]
pub struct MongoCatalogRepo
{
    client: Client
}

impl MongoCatalogRepo{
    pub fn new(client: Client) -> Self{
        Self{
            client
        }
    }
    
    pub async fn import_perm_relationships(&self) -> ServiceResult<()>
    {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");

        let db = self.client.database(database_name.as_str()) ;
        let coll = db.collection::<Document>("relationship");
        coll.delete_many(Document::new()).await
            .expect("Error al vaciar la colección 'relationship'");

        let file_path = "C:/Users/alorenzo/Proyectos-2/user/tests/fixtures/perms_relationship.json";
        let file_content = fs::read_to_string(file_path)
            .expect("Error al leer el archivo perms_relationship.json");

        let mut relationships: Vec<PermsRelationship> = serde_json::from_str(&file_content).unwrap();

        // Convert `Vec<u64>` to `Vec<u32>` during iteration
        for relationship in relationships.iter_mut() {
            relationship.perms = relationship
                .perms.clone() // Vec<u64>
                .into_iter()
                .map(|p| p.try_into().unwrap_or_else(|_| {
                    panic!("Error: No se pudo convertir {} a u32", p)
                })) // Vec<u32>
                .collect();
        }

        for relationship in relationships {
            let bson_doc = to_bson(&relationship)
                .expect("Error al convertir PermsRelationship a BSON")
                .as_document()
                .expect("Error al convertir BSON a Documento")
                .to_owned();

            coll.insert_one(bson_doc).await
                .expect("Error al insertar el documento en MongoDB");
        }

        Ok(())


    }
    
   

    pub async fn fetch_perm_relationships(&self) -> Result<HashMap<Role, Vec<u32>>, ServiceError> {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");

        let db = self.client.database(database_name.as_str());
        let relationship_coll = db.collection::<Document>("relationship");

        let mut relationship_map = HashMap::new();
        let filter = doc! {};
        let mut cursor = relationship_coll
            .find(filter)
            .await
            .map_err(|_| ServiceError::RelationalNotFound)?;

        while let Some(relational_doc) = cursor.try_next()
            .await
            .map_err(|_| ServiceError::RelationalDocumentNotFound)?
        {
            
            let role = relational_doc.get_str("role")
                .map(|r| r.to_string()) 
                .map_err(|_| ServiceError::RelationalDeserializeError)?;

            let permissions = relational_doc.get_array("perms")
                .map_err(|_| ServiceError::RelationalDeserializeError)?
                .iter()
                .map(|p| {
                    match p.as_i64() {
                        Some(value) => value.try_into().unwrap_or_else(|_| {
                            panic!("Error: No se pudo convertir {} a u32", value)
                        }),
                        None => panic!("Error: Permiso no era un Int64"),
                    }
                })
                .collect::<Vec<u32>>();
            
            let role_enum: Role = role.parse().unwrap();
            relationship_map
                .entry(role_enum)
                .or_insert_with(Vec::new)
                .extend(permissions);
        }

        Ok(relationship_map)
    }


}