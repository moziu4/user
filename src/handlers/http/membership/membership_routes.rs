use std::sync::Arc;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use crate::context::Context;
use crate::core::{
    domain::membership::membership_commands::MembershipCommand,
    operation::membership_ops::MembershipOps,
};
use crate::data::access::membership_repo::MongoMembershipRepo;

pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/membership")
            .route("/new", web::post().to(handle_command))
            .route("/by-user/{user}", web::get().to(get_membership_by_user))
            .route("/{id}", web::get().to(get_membership))
            .route("/{id}/command", web::post().to(handle_command_with_id))
    );
}

async fn handle_command(_req: HttpRequest, context: web::Data<Arc<Context>>, payload: web::Json<MembershipCommand>) -> impl Responder {
    let coll = context.get_ref().get_collection("memberships");
    let repo = MongoMembershipRepo::new(coll);
    let ops = MembershipOps::new(&repo, context.get_ref());

    match ops.execute_command(None, payload.into_inner()).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn handle_command_with_id(path: web::Path<String>, _req: HttpRequest, context: web::Data<Arc<Context>>, payload: web::Json<MembershipCommand>) -> impl Responder {
    let membership_id = path.into_inner();
    let coll = context.get_ref().get_collection("memberships");
    let repo = MongoMembershipRepo::new(coll);
    let ops = MembershipOps::new(&repo, context.get_ref());

    match ops.execute_command(Some(&membership_id), payload.into_inner()).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn get_membership_by_user(path: web::Path<String>, context: web::Data<Arc<Context>>) -> impl Responder {
    let user_id = path.into_inner();
    let coll = context.get_ref().get_collection("memberships");
    let repo = MongoMembershipRepo::new(coll);
    let ops = MembershipOps::new(&repo, context.get_ref());

    match ops.get_membership_for_user(&user_id).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn get_membership(path: web::Path<String>, context: web::Data<Arc<Context>>) -> impl Responder {
    let membership_id = path.into_inner();
    let coll = context.get_ref().get_collection("memberships");
    let repo = MongoMembershipRepo::new(coll);
    let ops = MembershipOps::new(&repo, context.get_ref());

    match ops.get_membership(&membership_id).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
