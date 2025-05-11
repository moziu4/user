use std::sync::Arc;

use actix_web::{web, web::Json, HttpResponse, Responder};

use crate::{
    context::Context,
    core::{domain::auth::auth_type::AuthLogin},
};
use crate::core::operation::auth_ops::AuthOps;

pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(web::scope("/api/auth").route("/login", web::post().to(login)));
}

async fn login(context: web::Data<Arc<Context>>, payload: Json<AuthLogin>) -> impl Responder
{
    let auth_repo = context.get_ref().get_auth_repo();
    let auth_ops = AuthOps::new(&auth_repo);
    match auth_ops.do_login(payload.into_inner())
                      .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
