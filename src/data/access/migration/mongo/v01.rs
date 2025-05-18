use std::{env, fs};
use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use dotenv::dotenv;
use mongodb::{error::Error as MongoError, Database};
use mongodb::bson::{doc, to_bson, Document};
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::perm_type::PermsRelationship;
use crate::core::domain::user::user_type::NewUser;
use crate::core::operation::user_ops::UserOps;
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
        let catalogs_path = env::var("CATALOGS_PATH").expect("Path of catalog not defined");
        let file_path = format!("{}/perms_relationship.json", catalogs_path);

        let file_content = fs::read_to_string(file_path)
            .expect("Error al leer el archivo perms_relationship.json");
        let relationships: Vec<PermsRelationship> = serde_json::from_str(&file_content).unwrap();
        for relationship in relationships {
            let coll = db.collection::<mongodb::bson::Document>("relationship");
            let bson_doc = to_bson(&relationship)
                .expect("Error al convertir PermsRelationship a BSON")
                .as_document()
                .expect("Error al convertir BSON a Document")
                .to_owned();

            coll.insert_one(bson_doc).await?;
        }

        Ok(())
    }

    async fn create_admin_user(&self, db: &Database) -> Result<(), MongoError> {
        let file_path = "C:/Users/alorenzo/Proyectos-2/user/tests/fixtures/user_admin.json"; 
        let file_content = fs::read_to_string(file_path)
            .expect("Error al leer el archivo user_admin.json");
        let user_data: Document = serde_json::from_str(&file_content)
            .expect("Error al deserializar el JSON de usuario administrador");

        let mut user_data_cleaned = user_data.clone();
        user_data_cleaned.remove("password");
        let user_coll = db.collection::<Document>("users");
        let insert_result = user_coll
            .insert_one(user_data_cleaned)
            .await
            .expect("Error al insertar el usuario en la base de datos");

        let user_id = insert_result
            .inserted_id
            .as_object_id()
            .expect("No se pudo obtener el ObjectId del usuario");

        let role = Role::SuperAdmin;
        let relationship_coll = db.collection::<Document>("relationship");
        let relationship_doc = relationship_coll
            .find_one(doc! { "role": role.to_string() })
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
        
        
        let auth_coll = db.collection::<Document>("auth");
        let auth_doc = doc! {
            "user_id": user_id,
            "username": user_data.get_str("username").expect("Falta el campo 'username'"),
            "email": user_data.get_str("email").expect("Falta el campo 'email'"),
            "password": hashed_password,
            "roles": role.to_string(),
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




