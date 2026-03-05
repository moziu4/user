use std::collections::HashMap;
use std::{env, fs};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, to_bson, Document};
use mongodb::{Client};
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::PermsRelationship;
use crate::error::{ServiceError, ServiceResult};

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

    pub async fn import_perms(&self) -> ServiceResult<()>
    {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "C:/Users/alorenzo/Proyectos-2/user/tests/fixtures".to_string());

        let db = self.client.database(database_name.as_str());
        let coll = db.collection::<Document>("perms");
        coll.delete_many(doc! {}).await
            .expect("Error al vaciar la colección 'perms'");

        let file_path = format!("{}/perms.json", catalogs_path);
        let file_content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Error al leer el archivo {}", file_path));

        // Note: Perm struct from domain has _id: Option<PermID>, name, description.
        // The JSON has id: u32, name, description.
        // We'll use a local struct or deserialize to a value and map it.
        #[derive(serde::Deserialize)]
        struct PermJson {
            id: u32,
            name: String,
            description: String,
        }

        let perms_json: Vec<PermJson> = serde_json::from_str(&file_content).unwrap();

        for p in perms_json {
            let doc = doc! {
                "id": p.id,
                "name": p.name,
                "description": p.description,
            };
            coll.insert_one(doc).await
                .expect("Error al insertar el permiso en MongoDB");
        }

        Ok(())
    }

    pub async fn import_perm_relationships(&self) -> ServiceResult<()>
    {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "C:/Users/alorenzo/Proyectos-2/user/tests/fixtures".to_string());

        let db = self.client.database(database_name.as_str()) ;
        let coll = db.collection::<Document>("relationship");
        coll.delete_many(doc! {}).await
            .expect("Error al vaciar la colección 'relationship'");

        // Try both perms_relationship.json and relationship.json as requested
        let mut file_path = format!("{}/relationship.json", catalogs_path);
        if !fs::metadata(&file_path).is_ok() {
            file_path = format!("{}/perms_relationship.json", catalogs_path);
        }

        let file_content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Error al leer el archivo {}", file_path));

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