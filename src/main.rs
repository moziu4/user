use std::{env, io, sync::Arc};
use actix_cors::Cors;
use actix_web::{
    dev::RequestHead,
    http::header::{self, HeaderValue},
    web,
    App,
    HttpServer,
};
use futures_util::FutureExt;
use clap::Parser;
use user::{context::Context, db::connect_to_db, handlers::http};
use env_logger::Env;
use user::core::operation::perms_ops::{PermOps};
use user::core::operation::user_ops::{UserOps};
use user::handlers::commands::Commands;
use user::handlers::commands::permsrelationship_commands::run_create_perms_relationship_command;
use user::handlers::commands::user_command::run_intern_user_command;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Inicializar el logger
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // Conectar a MongoDB
    let client = connect_to_db().await;
    let context = Arc::new(Context::new(client));

    let user_repo = context.get_user_repo();
    let auth_repo = context.get_auth_repo();
    let perm_repo = context.get_perm_repo();
    let user_ops = Arc::new(
        UserOps::new(&user_repo, perm_repo.as_ref(), auth_repo.as_ref(), &context).await,
    );
    let perms_ops = PermOps::new(&perm_repo, &context);
    

    // Parsea los comandos
    let args: Vec<String> = env::args().collect();
    let command_was_provided = args.len() > 1;

    if command_was_provided {
        match Commands::try_parse_from(&args) {
            Ok(command) => {
                match command {
                    Commands::InternNewUser => {
                        println!("Ejecutando comando InternNewUser...");
                        run_intern_user_command(
                            user_ops,
                            env::current_dir()
                                .unwrap()
                                .join("tests/fixtures/user_admin.json")
                                .to_str()
                                .unwrap(),
                        )
                            .await;
                    }
                    Commands::CreatePermsRelationship => {
                        println!("Ejecutando comando CreatePermsRelationship...");
                        run_create_perms_relationship_command(
                            &perms_ops,
                            env::current_dir()
                                .unwrap()
                                .join("tests/fixtures/perms_relationship.json")
                                .to_str()
                                .unwrap(),
                        )
                            .await;
                    }
                }
                println!("Comando ejecutado exitosamente.");
                return Ok(()); // Salir después de ejecutar el comando
            }
            Err(e) => {
                eprintln!("Error procesando el comando: {}", e);
                return Ok(()); // Mostrar el error y salir
            }
        }
    }

    // Si no hay comandos, levantar el servidor
    println!("Ningún comando proporcionado. Ejecutando únicamente el servidor...");

    let bind_address = env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:2030".to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(context.clone()))
            .configure(http::users::user_routes::config)
            .configure(http::auth::auth_routes::config)  
    })
        .bind(bind_address)?
        .run()
        .await
}
