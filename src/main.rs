use std::{env, io, sync::Arc};
use actix_cors::Cors;
use actix_web::{
    dev::RequestHead,
    http::header::{self, HeaderValue},
    web,
    App,
    HttpServer,
};

use user::{context::Context, db::connect_to_db, handlers::http};
use env_logger::Env;

use user::data::access::migration::{ MigrationContext};
use user::data::access::migration::mongo::migrate_mongo;

#[actix_web::main]
async fn main() -> io::Result<()>
{
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let client = connect_to_db().await;
    let context = Arc::new(Context::new(client.clone()));
    let migration_context = MigrationContext{ client: client.clone()};
    match migrate_mongo(migration_context).await {
        Ok(applied) => {
            println!("Migraciones completadas. Total migraciones aplicadas: {}", applied);
        }
        Err(err) => {
            eprintln!("Error al ejecutar las migraciones: {:?}", err);
            std::process::exit(1); // Salida del programa si hay un error crítico
        }
    }



    HttpServer::new(move || {
        App::new().wrap(Cors::default().allowed_origin_fn(|origin: &HeaderValue, _req_head: &RequestHead| {
            if let Ok(origin_str) = origin.to_str()
            {
                origin_str == env::var("URL_FRONT_DEV").unwrap()
                    || origin_str == env::var("URL_FRONT").unwrap()
            }
            else
            {
                false
            }
        })
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .max_age(3600))
            .app_data(web::Data::new(context.clone()))
            .configure(http::users::user_routes::config)
            .configure(http::auth::auth_routes::config)
            .configure(http::catalogs::catalog_routes::config)
    }).bind(env::var("HTTP_BIND").unwrap().to_string())?
        .run()
        .await
}
