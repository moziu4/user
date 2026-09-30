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
use user::handlers::message::nats_service::connect_nats;

#[actix_web::main]
async fn main() -> io::Result<()>
{
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let client = connect_to_db().await;
    let nats_client = connect_nats().await;
    let context = Arc::new(Context::new(client.clone(), nats_client.clone()));
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

    let nats_client_for_subscription = nats_client.clone();
    let context_for_sub = context.clone();
    if let Some(nats_c) = nats_client_for_subscription {
        actix_web::rt::spawn(async move {
            user::handlers::message::nats_service::NatsService::subscribe_membership_events(nats_c, context_for_sub).await;
        });
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
            .configure(http::membership::membership_routes::config)
    }).bind(env::var("HTTP_BIND").unwrap().to_string())?
        .run()
        .await
}
