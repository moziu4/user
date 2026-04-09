use std::{env, fs};
use std::collections::HashMap;
use std::str::FromStr;
use async_trait::async_trait;
use bcrypt::{hash};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use mongodb::{error::Error as MongoError, Database};
use mongodb::bson::{doc, to_bson, Document};
use serde::Deserialize;
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::{PermsRelationship, PermsRelationshipDTO};
use crate::data::access::migration::MigrationContext;
use crate::data::access::migration::Migration;

pub struct Migration001;

#[async_trait]
impl Migration for Migration001 {
    fn name(&self) -> &'static str {
        "create_admin_and_relationships"
    }

    async fn up(&self, context: &MigrationContext) -> Result<(), MongoError> {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Variable isn't found: MONGO_DATABASE");
        
        let db = context.client.database(database_name.as_str());

        Migration001::create_relationships(self, &db).await?;
        Migration001::create_admin_user(self, &db).await?;

        Ok(())
    }
}

impl Migration001 {
    async fn create_relationships(&self, db: &Database) -> Result<(), MongoError> {
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "tests/fixtures".to_string());
        
        let file_content = if catalogs_path.starts_with("http") {
            let mut content = None;
            let names = vec!["relationship.json", "perms_relationship.json"];
            let client = reqwest::Client::builder()
                .user_agent("user-service")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            for name in names {
                let url = format!("{}/{}", catalogs_path, name);
                if let Ok(resp) = client.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(text) = resp.text().await {
                            content = Some(text);
                            break;
                        }
                    }
                }
            }
            content
        } else {
            None
        };

        let file_content = if let Some(content) = file_content {
            content
        } else {
            // Intentar encontrar el archivo en varias rutas posibles
            let mut base_paths = vec![
                "tests/fixtures".to_string(), 
                "/opt".to_string(),
                "/opt/catalogs".to_string(),
                "catalogs/user-messages".to_string(),
                "../catalogs/user-messages".to_string(),
                "C:/Users/alorenzo/Proyectos-2/catalogs/user-messages".to_string(),
            ];
            
            if !catalogs_path.starts_with("http") {
                base_paths.insert(0, catalogs_path.clone());
            }
            
            let mut content = None;

            for base in base_paths {
                let paths_to_try = vec![
                    format!("{}/relationship.json", base),
                    format!("{}/perms_relationship.json", base),
                ];

                for path in paths_to_try {
                    if let Ok(c) = fs::read_to_string(&path) {
                        content = Some(c);
                        break;
                    }
                }
                if content.is_some() { break; }
            }
            content.expect(&format!(
                "Error: No se pudo encontrar relationship.json o perms_relationship.json en {} ni en rutas locales", 
                catalogs_path
            ))
        };

        let mut relationships_dto: Vec<PermsRelationshipDTO> = serde_json::from_str(&file_content)
            .expect("Error al deserializar el JSON de relaciones");

        // Cargar mapa de permisos para convertir nombres a IDs
        let perms_map = self.get_perms_map(db).await?;
        
        let coll = db.collection::<Document>("relationship");

        for dto in relationships_dto {
            let role = Role::from_str(&dto.role).unwrap_or_else(|_| {
                    // Fallback para mapeo manual
                    match dto.role.as_str() {
                        "SuperAdmin" => Role::SuperAdmin,
                        "AgencyOwner" => Role::AgencyOwner,
                        "AgencyAdmin" => Role::AgencyAdmin,
                        "AgencyMember" => Role::AgencyMember,
                        "TenantAdmin" => Role::TenantAdmin,
                        "Editor" => Role::Editor,
                        "Client" => Role::Client,
                        "Guest" => Role::Guest,
                        _ => panic!("Rol desconocido: {}", dto.role)
                    }
                });

            let relationship = PermsRelationship {
                id: dto.id,
                role: role,
                perms: dto.perms,
            };

            let filter = doc! { "role": relationship.role as u32 };
            let count = coll.count_documents(filter).await?;
            if count == 0 {
                let bson_doc = to_bson(&relationship)
                    .expect("Error al convertir PermsRelationship a BSON")
                    .as_document()
                    .expect("Error al convertir BSON a Document")
                    .clone();

                coll.insert_one(bson_doc).await?;
            }
        }

        Ok(())
    }

    async fn create_admin_user(&self, db: &Database) -> Result<(), MongoError> {
        let catalogs_path = env::var("CATALOGS_PATH")
            .unwrap_or_else(|_| "tests/fixtures".to_string());
        
        let file_content = if catalogs_path.starts_with("http") {
            let url = format!("{}/user_admin.json", catalogs_path);
            let mut content = None;
            let client = reqwest::Client::builder()
                .user_agent("user-service")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(text) = resp.text().await {
                        content = Some(text);
                    }
                }
            }
            content
        } else {
            None
        };

        let file_content = if let Some(content) = file_content {
            content
        } else {
            let mut file_path = format!("{}/user_admin.json", catalogs_path);
            let mut content = None;
            
            if !catalogs_path.starts_with("http") && fs::metadata(&file_path).is_ok() {
                content = fs::read_to_string(&file_path).ok();
            }

            if content.is_none() {
                let fallback_paths = vec![
                    "tests/fixtures/user_admin.json", 
                    "/opt/user_admin.json",
                    "/opt/catalogs/user_admin.json",
                    "catalogs/user-messages/user_admin.json",
                    "../catalogs/user-messages/user_admin.json",
                    "C:/Users/alorenzo/Proyectos-2/catalogs/user-messages/user_admin.json"
                ];
                for fallback in fallback_paths {
                    if fs::metadata(fallback).is_ok() {
                        content = fs::read_to_string(fallback).ok();
                        break;
                    }
                }
            }

            content.expect(&format!("Error al leer el archivo user_admin.json desde {} o rutas locales", catalogs_path))
        };

        let user_data: Document = serde_json::from_str(&file_content)
            .expect("Error al deserializar el JSON de usuario administrador");

        let username = user_data.get_str("username").expect("Falta el campo 'username'");
        let email = user_data.get_str("email").expect("Falta el campo 'email'");

        let auth_coll = db.collection::<Document>("auth");
        let existing_auth = auth_coll.find_one(doc! { "username": username }).await?;
        
        if existing_auth.is_some() {
            println!("El usuario administrador '{}' ya existe. Saltando creación.", username);
            return Ok(());
        }

        let mut user_data_cleaned = user_data.clone();
        user_data_cleaned.remove("password");
        let user_coll = db.collection::<Document>("users");
        
        // Check if user already exists in users collection too
        let existing_user = user_coll.find_one(doc! { "username": username }).await?;
        let user_id = if let Some(u) = existing_user {
            u.get_object_id("_id").expect("No se pudo obtener el ObjectId del usuario existente")
        } else {
            let insert_result = user_coll
                .insert_one(user_data_cleaned)
                .await
                .expect("Error al insertar el usuario en la base de datos");

            insert_result
                .inserted_id
                .as_object_id()
                .expect("No se pudo obtener el ObjectId del usuario insertado")
        };

        let role = Role::SuperAdmin;
        let relationship_coll = db.collection::<Document>("relationship");
        
        // Buscamos por el nombre del rol (String) o por el ID si se guardó como tal
        // Dado que PermsRelationship usa el enum Role, serde por defecto lo serializa como el nombre de la variante.
        let role_name = format!("{:?}", role);
        let filter = doc! { 
            "$or": [
                { "role": role_name },
                { "role": role as u32 },
                { "id": 1 } // SuperAdmin en el catálogo externo es ID 1
            ]
        };
        
        let relationship_doc = relationship_coll
            .find_one(filter.clone())
            .await?;
            
        let perms = if let Some(doc) = relationship_doc {
             doc.get_array("perms")
                .expect("No se encontró el campo `perms` en la relación")
                .iter()
                .map(|perm| {
                    if let Some(val) = perm.as_str() {
                        val.to_string()
                    } else {
                        panic!("Permiso no válido: se esperaba un string");
                    }
                })
                .collect::<Vec<String>>()
        } else {
            // Fallback: Si no se encontró en la colección, intentar cargar del mapa local
            // (que se construyó a partir del JSON en create_relationships)
            let perms_map = self.get_perms_map(db).await?;
            if perms_map.is_empty() {
                 panic!("No se encontraron permisos para el rol SuperAdmin en la BD ni en el catálogo");
            }
            
            // Aquí perms_map mapea NOMBRE_PERMISO -> ID.
            // Pero nosotros queremos los NOMBRES de los permisos para el rol SuperAdmin.
            // Como create_relationships ya corrió, la colección 'relationship' DEBERÍA estar poblada.
            // Si no está, algo falló en create_relationships o el filtro es incorrecto.
            
            // Vamos a mostrar qué roles hay en la colección para depurar
            let mut cursor = relationship_coll.find(doc! {}).await?;
            while let Some(r_doc) = cursor.try_next().await? {
                println!("Relación encontrada en BD: {:?}", r_doc);
            }
            
            panic!("No se encontraron permisos para el rol SuperAdmin (Nombre: {}, ID: {})", format!("{:?}", role), role as u32);
        };

        let plain_password = user_data
            .get_str("password")
            .expect("Falta el campo `password` en el JSON");
        let hashed_password = hash(plain_password, 10)
            .expect("Error al hashear el password");
        
        let auth_doc = doc! {
            "user_id": user_id,
            "username": username,
            "email": email,
            "password": hashed_password,
            "role_id": role as u32,
            "permissions": perms.clone(),
            "granted_permissions": perms,
            "denied_permissions": []
        };
        
        auth_coll
            .insert_one(auth_doc)
            .await
            .expect("Error al insertar el documento de autenticación en la base de datos");

        println!("Usuario administrador creado e insertado con éxito.");
        Ok(())
    }

    async fn get_perms_map(&self, db: &Database) -> Result<HashMap<String, u32>, MongoError> {
        let coll = db.collection::<Document>("perms");
        let mut cursor = coll.find(doc! {}).await?;
        let mut map = HashMap::new();
        while let Some(doc) = cursor.try_next().await? {
            if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i32("id")) {
                map.insert(name.to_string(), id as u32);
            } else if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i64("id")) {
                map.insert(name.to_string(), id as u32);
            }
        }
        
        // Si el mapa está vacío, intentar cargar desde el archivo perms.json
        if map.is_empty() {
             let catalogs_path = env::var("CATALOGS_PATH")
                .unwrap_or_else(|_| "tests/fixtures".to_string());
             
             let perms_content = if catalogs_path.starts_with("http") {
                 let url = format!("{}/perms.json", catalogs_path);
                 if let Ok(r) = reqwest::get(&url).await {
                     r.text().await.ok()
                 } else {
                     None
                 }
             } else {
                 fs::read_to_string(format!("{}/perms.json", catalogs_path)).ok()
             };

             if let Some(content) = perms_content {
                 #[derive(Deserialize)]
                 struct PermItem { id: u32, name: String }
                 if let Ok(items) = serde_json::from_str::<Vec<PermItem>>(&content) {
                     for item in items {
                         map.insert(item.name, item.id);
                     }
                 }
             }
        }

        Ok(map)
    }
}




