use std::collections::HashMap;
use std::{env, fs};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, to_bson, Document};
use mongodb::{Client};
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::{PermsRelationship, DocumentType, RoleInfo, PermsRelationshipDTO};
use std::path::Path;
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
            .unwrap_or_else(|_| "tests/fixtures".to_string());

        let db = self.client.database(database_name.as_str());
        let coll = db.collection::<Document>("perms");
        coll.delete_many(doc! {}).await
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al vaciar la colección 'perms': {}", e)))?;

        let file_content = if catalogs_path.starts_with("http") {
            let url = format!("{}/perms.json", catalogs_path);
            println!("Descargando perms desde: {}", url);
            let client = reqwest::Client::builder()
                .user_agent("user-service")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.text().await {
                        Ok(text) => Some(text),
                        Err(e) => {
                            println!("Error al leer respuesta de perms: {}. Intentando fallback...", e);
                            None
                        }
                    }
                }
                Ok(resp) => {
                    println!("Error de respuesta al descargar perms ({}): {}. Intentando fallback...", resp.status(), catalogs_path);
                    None
                }
                Err(e) => {
                    println!("Error al conectar con GitHub para perms: {}. Intentando fallback...", e);
                    None
                }
            }
        } else {
            None
        };

        let file_content = if let Some(content) = file_content {
            content
        } else {
            let mut file_path = format!("{}/perms.json", catalogs_path);
            let mut content = None;
            
            // Intentar con la ruta actual o si es URL fallida, usar rutas por defecto
            if !catalogs_path.starts_with("http") && Path::new(&file_path).exists() {
                content = fs::read_to_string(&file_path).ok();
            }

            if content.is_none() {
                println!("Buscando perms.json en rutas alternativas...");
                let fallbacks = vec![
                    "tests/fixtures/perms.json", 
                    "/opt/perms.json", 
                    "/opt/catalogs/perms.json",
                    "catalogs/user-messages/perms.json",
                    "../catalogs/user-messages/perms.json",
                    "C:/Users/alorenzo/Proyectos-2/catalogs/user-messages/perms.json"
                ];
                for f in fallbacks {
                    if Path::new(f).exists() {
                        println!("Archivo perms.json encontrado en: {}", f);
                        content = fs::read_to_string(f).ok();
                        break;
                    }
                }
            }
            content.ok_or_else(|| ServiceError::CatalogFileError(format!("No se pudo encontrar perms.json en {} ni en rutas locales", catalogs_path)))?
        };

        // Note: Perm struct from domain has _id: Option<PermID>, name, description.
        // The JSON has id: u32, name, description.
        // We'll use a local struct or deserialize to a value and map it.
        #[derive(serde::Deserialize)]
        struct PermJson {
            id: u32,
            name: String,
            description: String,
        }

        let perms_json: Vec<PermJson> = serde_json::from_str(&file_content)
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al parsear el JSON de perms: {}", e)))?;

        for p in perms_json {
            let doc = doc! {
                "id": p.id,
                "name": p.name,
                "description": p.description,
            };
            coll.insert_one(doc).await
                .map_err(|e| ServiceError::CatalogFileError(format!("Error al insertar el permiso en MongoDB: {}", e)))?;
        }

        Ok(())
    }

    pub async fn import_perm_relationships(&self) -> ServiceResult<()>
    {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "tests/fixtures".to_string());

        let db = self.client.database(database_name.as_str()) ;
        let coll = db.collection::<Document>("relationship");
        coll.delete_many(doc! {}).await
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al vaciar la colección 'relationship': {}", e)))?;

        let file_content = if catalogs_path.starts_with("http") {
            let mut content = None;
            let names = vec!["relationship.json", "perms_relationship.json"];
            let client = reqwest::Client::builder()
                .user_agent("user-service")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            for name in names {
                let url = format!("{}/{}", catalogs_path, name);
                println!("Descargando relaciones desde: {}", url);
                match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(text) = resp.text().await {
                            content = Some(text);
                            break;
                        }
                    }
                    _ => continue,
                }
            }
            content
        } else {
            None
        };

        let file_content = if let Some(content) = file_content {
            content
        } else {
            // Try multiple paths and filenames
            let mut content = None;
            let mut bases = vec![
                "tests/fixtures".to_string(), 
                "/opt".to_string(),
                "/opt/catalogs".to_string(),
                "catalogs/user-messages".to_string(),
                "../catalogs/user-messages".to_string(),
                "C:/Users/alorenzo/Proyectos-2/catalogs/user-messages".to_string(),
            ];
            
            if !catalogs_path.starts_with("http") {
                bases.insert(0, catalogs_path.clone());
            }

            for base in bases {
                let to_try = vec![
                    format!("{}/relationship.json", base),
                    format!("{}/perms_relationship.json", base),
                ];
                for path in to_try {
                    if let Ok(c) = fs::read_to_string(&path) {
                        println!("Archivo de relaciones encontrado en: {}", path);
                        content = Some(c);
                        break;
                    }
                }
                if content.is_some() { break; }
            }
            content.ok_or_else(|| {
                ServiceError::CatalogFileError(format!("Error: No se pudo encontrar el archivo de relaciones en {} ni en rutas por defecto", catalogs_path))
            })?
        };

        let mut perms_relationships_dto: Vec<PermsRelationshipDTO> = serde_json::from_str(&file_content)
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al parsear el JSON de relaciones: {}", e)))?;

        // Cargar mapa de permisos para convertir nombres a IDs
        let perms_map = self.get_perms_map(&db).await?;

        for dto in perms_relationships_dto {
            let role = match dto.role.as_str() {
                "SuperAdmin" => Role::SuperAdmin,
                "AgencyOwner" => Role::AgencyOwner,
                "AgencyAdmin" => Role::AgencyAdmin,
                "AgencyMember" => Role::AgencyMember,
                "TenantAdmin" => Role::TenantAdmin,
                "Editor" => Role::Editor,
                "Client" => Role::Client,
                "Guest" => Role::Guest,
                _ => {
                   // Intentar usar FromStr si está disponible
                   dto.role.parse::<Role>().unwrap_or(Role::Guest)
                }
            };

            let relationship = PermsRelationship {
                id: dto.id,
                role: role,
                perms: dto.perms,
            };

            let bson_doc = to_bson(&relationship)
                .map_err(|e| ServiceError::CatalogFileError(format!("Error al convertir PermsRelationship a BSON: {}", e)))?
                .as_document()
                .ok_or_else(|| ServiceError::CatalogFileError("Error al convertir BSON a Documento".to_string()))?
                .clone();

            coll.insert_one(bson_doc).await
                .map_err(|e| ServiceError::CatalogFileError(format!("Error al insertar el documento de relación en MongoDB: {}", e)))?;
        }

        Ok(())


    }
    
   


    pub async fn import_document_types(&self) -> ServiceResult<()>
    {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "tests/fixtures".to_string());

        let db = self.client.database(database_name.as_str());
        let coll = db.collection::<Document>("document_types");
        coll.delete_many(doc! {}).await
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al vaciar la colección 'document_types': {}", e)))?;

        let file_content = if catalogs_path.starts_with("http") {
            let url = format!("{}/documentType.json", catalogs_path);
            println!("Descargando documentType desde: {}", url);
            let client = reqwest::Client::builder()
                .user_agent("user-service")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    resp.text().await.ok()
                }
                _ => None
            }
        } else {
            None
        };

        let file_content = if let Some(content) = file_content {
            content
        } else {
            let mut content = None;
            let fallbacks = vec![
                format!("{}/documentType.json", catalogs_path),
                "tests/fixtures/documentType.json".to_string(),
                "/opt/documentType.json".to_string(),
                "/opt/catalogs/documentType.json".to_string(),
                "catalogs/user-messages/documentType.json".to_string(),
                "../catalogs/user-messages/documentType.json".to_string(),
                "C:/Users/alorenzo/Proyectos-2/catalogs/user-messages/documentType.json".to_string()
            ];
            for f in fallbacks {
                if Path::new(&f).exists() {
                    content = fs::read_to_string(f).ok();
                    break;
                }
            }
            content.ok_or_else(|| ServiceError::CatalogFileError(format!("No se pudo encontrar documentType.json")))?
        };

        let doc_types: Vec<DocumentType> = serde_json::from_str(&file_content)
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al parsear el JSON de documentType: {}", e)))?;

        for dt in doc_types {
            let bson_doc = to_bson(&dt)
                .map_err(|e| ServiceError::CatalogFileError(format!("Error al convertir DocumentType a BSON: {}", e)))?
                .as_document()
                .ok_or_else(|| ServiceError::CatalogFileError("Error al convertir BSON a Documento".to_string()))?
                .to_owned();

            coll.insert_one(bson_doc).await
                .map_err(|e| ServiceError::CatalogFileError(format!("Error al insertar el tipo de documento en MongoDB: {}", e)))?;
        }

        Ok(())
    }

    pub async fn fetch_document_types(&self) -> Result<Vec<DocumentType>, ServiceError> {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");

        let db = self.client.database(database_name.as_str());
        let coll = db.collection::<Document>("document_types");

        let mut cursor = coll.find(doc! {}).await
            .map_err(|_| ServiceError::RelationalNotFound)?;

        let mut doc_types = Vec::new();
        while let Some(doc) = cursor.try_next().await
            .map_err(|_| ServiceError::RelationalDocumentNotFound)?
        {
            let doc_type: DocumentType = mongodb::bson::from_document(doc)
                .map_err(|_| ServiceError::RelationalDeserializeError)?;
            doc_types.push(doc_type);
        }

        Ok(doc_types)
    }

    pub async fn fetch_roles(&self) -> Result<Vec<RoleInfo>, ServiceError> {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");

        let db = self.client.database(database_name.as_str());
        let coll = db.collection::<Document>("relationship");

        let mut cursor = coll.find(doc! {}).await
            .map_err(|_| ServiceError::RelationalNotFound)?;

        let mut roles = Vec::new();
        while let Some(doc) = cursor.try_next().await
            .map_err(|_| ServiceError::RelationalDocumentNotFound)?
        {
            let role_info: RoleInfo = mongodb::bson::from_document(doc)
                .map_err(|_| ServiceError::RelationalDeserializeError)?;
            roles.push(role_info);
        }

        Ok(roles)
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
            
            let role_id = relational_doc.get_i32("role")
                .map(|r| r as u32) 
                .or_else(|_| relational_doc.get_i64("role").map(|r| r as u32))
                .map_err(|_| ServiceError::RelationalDeserializeError)?;

            let permissions = relational_doc.get_array("perms")
                .map_err(|_| ServiceError::RelationalDeserializeError)?
                .iter()
                .map(|p| {
                    match p.as_i64() {
                        Some(value) => value.try_into().unwrap_or_else(|_| {
                            panic!("Error: No se pudo convertir {} a u32", value)
                        }),
                        None => match p.as_i32() {
                            Some(value) => value as u32,
                            None => panic!("Error: Permiso no era un número"),
                        }
                    }
                })
                .collect::<Vec<u32>>();
            
            let role_enum: Role = Role::from_id(role_id).unwrap_or(Role::Guest);
            relationship_map
                .entry(role_enum)
                .or_insert_with(Vec::new)
                .extend(permissions);
        }

        Ok(relationship_map)
    }

    async fn get_perms_map(&self, db: &mongodb::Database) -> ServiceResult<HashMap<String, u32>> {
        let coll = db.collection::<Document>("perms");
        let mut cursor = coll.find(doc! {}).await
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al buscar permisos: {}", e)))?;
        let mut map = HashMap::new();
        while let Some(doc) = cursor.try_next().await
            .map_err(|e| ServiceError::CatalogFileError(format!("Error al iterar permisos: {}", e)))? {
            if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i32("id")) {
                map.insert(name.to_string(), id as u32);
            } else if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i64("id")) {
                map.insert(name.to_string(), id as u32);
            }
        }
        Ok(map)
    }
}