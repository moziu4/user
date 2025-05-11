#[cfg(test)]
mod tests
{
    use std::{env, fs, sync::Arc};

    use user::{
        context::Context,
        core::{domain::user::user_type::NewUser, operation::user_ops::UserService},
    };
    use mongodb::{options::ClientOptions, Client};

    async fn create_test_context() -> Arc<Context>
    {
        let client_options =
            ClientOptions::parse("mongodb://dev:dev_pass@localhost:57017").await
                                                                          .expect("Error al configurar las opciones \
                                                                                   del cliente MongoDB");
        let client = Client::with_options(client_options).expect("Error al conectar a MongoDB de prueba");
        Arc::new(Context::new(client))
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

        match user_service.new_user(new_user).await
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
