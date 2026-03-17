use std::{env, fs};
use async_trait::async_trait;
use bcrypt::{hash};
use dotenv::dotenv;
use mongodb::{error::Error as MongoError, Database};
use mongodb::bson::{doc, to_bson, Document};
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::PermsRelationship;
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

        let relationships: Vec<PermsRelationship> = serde_json::from_str(&file_content)
            .expect("Error al deserializar el JSON de relaciones");
        let coll = db.collection::<Document>("relationship");

        for relationship in relationships {
            let filter = doc! { "role": (relationship.role as u32) };
            let count = coll.count_documents(filter).await?;
            if count == 0 {
                let mut bson_doc = to_bson(&relationship)
                    .expect("Error al convertir PermsRelationship a BSON")
                    .as_document()
                    .expect("Error al convertir BSON a Document")
                    .clone();

                // Asegurar que el campo 'role' en el documento insertado sea el ID numérico
                bson_doc.insert("role", relationship.role as u32);

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
        let relationship_doc = relationship_coll
            .find_one(doc! { "role": role.to_id() })
            .await?
            .expect("No se encontraron permisos para el rol SuperAdmin");

        let perms = relationship_doc
            .get_array("perms")
            .expect("No se encontró el campo `perms` en la relación")
            .iter()
            .map(|perm| {
                if let Some(val) = perm.as_i32() {
                    val as u32
                } else if let Some(val) = perm.as_i64() {
                    val as u32
                } else {
                    println!("Permiso no válido encontrado: {:?}", perm);
                    panic!("Permiso no válido: valor fuera de rango o formato no soportado");
                }
            })
            .collect::<Vec<u32>>();

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
            "role_id": role.to_id(),
            "permissions": perms
        };
        
        auth_coll
            .insert_one(auth_doc)
            .await
            .expect("Error al insertar el documento de autenticación en la base de datos");

        println!("Usuario administrador creado e insertado con éxito.");
        Ok(())
    }
}




