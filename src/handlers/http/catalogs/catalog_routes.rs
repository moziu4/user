use std::sync::Arc;
use actix_web::{web, HttpResponse, Responder};
use crate::context::Context;
use crate::core::operation::catalogs_ops::CatalogsOps;
use crate::data::catalog_importer::MongoCatalogRepo;

pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/catalogs")
            .route("/import", web::post().to(import_catalogs)
            ));
}

async fn import_catalogs(context: web::Data<Arc<Context>>) -> impl Responder
{
    let client = context.client.as_ref().clone();
    let repo = MongoCatalogRepo::new(client.clone());
    let auth_repo = context.get_ref().auth_repo.clone();
    
    let catalogs_ops = CatalogsOps::new(&repo, &auth_repo);
    match catalogs_ops.sync_catalogs()
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}