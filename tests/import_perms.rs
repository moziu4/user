use user::core::{domain::user::user_type::NewUser, operation::user_ops::UserService};

#[cfg(test)]
mod tests
{
    use std::{env, fs, sync::Arc};

    use user::{
        context::Context,
        core::{
            domain::{
                perm::perm_type::{Perms, PermsRelationship},
                user::user_type::NewUser,
            },
            operation::{perms_ops::PermsService, user_ops::UserService},
        },
        data::access::perms_repo::PermsRepositoryImpl,
    };
    use mongodb::{
        bson,
        bson::{doc, Document},
        options::ClientOptions,
        Client,
    };

    async fn create_test_context() -> Arc<Context>
    {
        let client_options =
            ClientOptions::parse("mongodb://dev:dev_pass@localhost:57017").await
                                                                          .expect("Error al configurar las opciones \
                                                                                   del cliente MongoDB");
        let client = Client::with_options(client_options).expect("Error al conectar a MongoDB de prueba");
        Arc::new(Context::new(client))
    }

    // #[tokio::test]
    async fn import_perms()
    {
        let current_dir = env::current_dir().expect("No se pudo obtener el directorio actual");

        let context = create_test_context().await;

        let perms_repo = Arc::new(PermsRepositoryImpl::new(context.client.clone()));

        let perms_service = PermsService { perms_repo };
        let db = context.client.database("mydb");

        db.collection::<Document>("perm")
          .drop()
          .await
          .expect("Error al eliminar la colección perm");


        let absolute_json_path = current_dir.join("tests/fixtures/perms.json");
        println!("Ruta absoluta al JSON: {}", absolute_json_path.display());

        let json_content = fs::read_to_string(&absolute_json_path).expect("No se pudo leer el archivo user_admin.json");

        let perms: Vec<Perms> =
            serde_json::from_str(&json_content).expect("No se pudo deserializar el JSON a Vec<Perms>");

        match perms_service.create_perms(perms.clone()).await
        {
            Ok(_) =>
            {
                println!("Permisos creados exitosamente.");

                let collection = context.client
                                        .database("mydb")
                                        .collection::<Document>("perm");
                for perm in perms
                {
                    let filter = doc! { "id": perm.id };
                    match collection.find_one(filter).await
                    {
                        Ok(Some(document)) =>
                        {
                            let inserted_perm: Perms =
                                bson::from_document(document).expect("No se pudo convertir el documento a Perms");
                            assert_eq!(inserted_perm.id, perm.id);
                            assert_eq!(inserted_perm.name, perm.name);
                            assert_eq!(inserted_perm.description, perm.description);
                            println!("Permiso verificado: {:?}", inserted_perm);
                        },
                        Ok(None) => panic!("Permiso no encontrado en la base de datos: {:?}", perm),
                        Err(err) => panic!("Error al buscar el permiso en la base de datos: {:?}", err),
                    }
                }
            },
            Err(err) => panic!("Error creando permisos: {:?}", err),
        }
    }

    #[tokio::test]
    async fn import_perms_relationship()
    {
        let current_dir = env::current_dir().expect("No se pudo obtener el directorio actual");

        let context = create_test_context().await;

        let perms_repo = Arc::new(PermsRepositoryImpl::new(context.client.clone()));

        let perms_service = PermsService { perms_repo };
        let db = context.client.database("mydb");

        // Drop the collection to start fresh
        db.collection::<Document>("relationship")
          .drop()
          .await
          .expect("Error al eliminar la colección perms_relationship");

        // Reading the JSON file
        let absolute_json_path = current_dir.join("tests/fixtures/perms_relationship.json");
        println!("Ruta absoluta al JSON: {}", absolute_json_path.display());

        let json_content =
            fs::read_to_string(&absolute_json_path).expect("No se pudo leer el archivo perms_relationship.json");

        // Deserialize the JSON into PermsRelationship
        let perms_relationships: Vec<PermsRelationship> =
            serde_json::from_str(&json_content).expect("No se pudo deserializar el JSON a Vec<PermsRelationship>");

        // Insert the perm relationships into the database
        match perms_service.create_perms_relationship(perms_relationships.clone())
                           .await
        {
            Ok(_) =>
            {
                println!("Relaciones de permisos creadas exitosamente.");

                let collection = db.collection::<Document>("relationship");
                for perm_relationship in perms_relationships
                {
                    let filter = doc! { "role": perm_relationship.role.to_string() };

                    // Verify the inserted document
                    match collection.find_one(filter).await
                    {
                        Ok(Some(document)) =>
                        {
                            let inserted_perm: PermsRelationship =
                                bson::from_document(document).expect("No se pudo convertir el documento a \
                                                                      PermsRelationship");
                            assert_eq!(inserted_perm.role, perm_relationship.role);
                            assert_eq!(inserted_perm.perms, perm_relationship.perms);
                            println!("Relación de permiso verificada: {:?}", inserted_perm);
                        },
                        Ok(None) =>
                        {
                            panic!("Relación de permiso no encontrada en la base de datos: {:?}", perm_relationship)
                        },
                        Err(err) => panic!("Error al buscar la relación de permiso en la base de datos: {:?}", err),
                    }
                }
            },
            Err(err) => panic!("Error creando relaciones de permisos: {:?}", err),
        }
    }

    #[tokio::test]
    async fn import_admin()
    {
        let current_dir = env::current_dir().expect("No se pudo obtener el directorio actual");
        println!("Directorio actual: {}", current_dir.display());

        let fixtures_path = current_dir.join("tests/fixtures");
        let paths = fs::read_dir(&fixtures_path).expect("No se pudo leer el directorio fixtures");
        for path in paths
        {
            println!("Encontrado archivo: {}", path.unwrap().path().display());
        }

        let context = create_test_context().await;

        let user_service = UserService { user_repo:  context.get_user_repo(),
                                         auth_repo:  context.get_auth_repo(),
                                         perms_repo: context.get_perm_repo(), };

        let absolute_json_path = current_dir.join("tests/fixtures/user_admin.json");
        println!("Ruta absoluta al JSON: {}", absolute_json_path.display());

        let json_content = fs::read_to_string(&absolute_json_path).expect("No se pudo leer el archivo user_admin.json");

        let new_user: NewUser = serde_json::from_str(&json_content).expect("No se pudo deserializar el JSON a UserNew");

        let command = "private".to_string();

        match user_service.intern_new_user(new_user).await
        {
            Ok(user) =>
            {
                assert_eq!(user.username, "admin");
                println!("Usuario admin creado exitosamente: {:?}", user)
            },
            Err(err) => panic!("Error creando usuario admin: {:?}", err),
        }
    }
}
