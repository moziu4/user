use std::sync::Arc;
use std::str::FromStr;
use actix_web::{web, HttpResponse, Responder};
use perms::UserID;
use serde::Deserialize;
use mongodb::bson::oid::ObjectId;
use crate::context::Context;
use crate::core::domain::fact::{
    fact_error::FactError,
    fact_type::{CreateFactDTO, FactStatus, FactType, UpdateFactStatusDTO},
};
use crate::core::operation::fact_ops::FactOps;
use crate::utils::domains_ids::FactID;

#[derive(Deserialize)]
pub struct FactQueryFilter {
    pub status: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/internal/users")
            .route("/{user_id}", web::get().to(get_user_profile))
            .route("/{user_id}/context", web::get().to(get_user_context))
            .route("/{user_id}/facts", web::get().to(get_user_facts))
            .route("/{user_id}/facts", web::post().to(create_user_fact))
            .route("/{user_id}/facts/{fact_type}", web::get().to(get_user_facts_by_type))
            .route("/facts/{fact_id}/status", web::put().to(update_fact_status))
            .route("/facts/{fact_id}/revoke", web::post().to(revoke_fact))
    );
}

fn parse_user_id(user_id_str: &str) -> Result<UserID, HttpResponse> {
    ObjectId::from_str(user_id_str)
        .map(UserID::from_object_id)
        .map_err(|_| HttpResponse::BadRequest().json("Invalid user ID format"))
}

async fn get_user_profile(
    path: web::Path<String>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let user_id_str = path.into_inner();
    let user_id = match parse_user_id(&user_id_str) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match context.get_ref().user_repo.fetch_by_id(user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}

async fn get_user_context(
    path: web::Path<String>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let user_id_str = path.into_inner();
    let user_id = match parse_user_id(&user_id_str) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let ops = FactOps::new(context.get_ref());

    match ops.get_user_context(&user_id).await {
        Ok(ctx) => HttpResponse::Ok().json(ctx),
        Err(FactError::UserNotFound) => HttpResponse::NotFound().json("User not found"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn get_user_facts(
    path: web::Path<String>,
    query: web::Query<FactQueryFilter>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let user_id_str = path.into_inner();
    let user_id = match parse_user_id(&user_id_str) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let status_filter = query.status.as_deref().and_then(|s| {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Some(FactStatus::Active),
            "SUPERSEDED" => Some(FactStatus::Superseded),
            "CONFLICTING" => Some(FactStatus::Conflicting),
            "REVOKED" => Some(FactStatus::Revoked),
            _ => None,
        }
    });

    let ops = FactOps::new(context.get_ref());

    match ops.get_facts_by_user(&user_id, status_filter.as_ref()).await {
        Ok(facts) => HttpResponse::Ok().json(facts),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn get_user_facts_by_type(
    path: web::Path<(String, String)>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let (user_id_str, fact_type_str) = path.into_inner();
    let user_id = match parse_user_id(&user_id_str) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let fact_type = FactType::from_str(&fact_type_str).unwrap();
    let ops = FactOps::new(context.get_ref());

    match ops.get_facts_by_type(&user_id, &fact_type).await {
        Ok(facts) => HttpResponse::Ok().json(facts),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn create_user_fact(
    path: web::Path<String>,
    payload: web::Json<CreateFactDTO>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let user_id_str = path.into_inner();
    let user_id = match parse_user_id(&user_id_str) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let mut dto = payload.into_inner();
    dto.user_id = user_id;

    let ops = FactOps::new(context.get_ref());

    match ops.create_fact(dto, None).await {
        Ok(fact) => HttpResponse::Created().json(fact),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn update_fact_status(
    path: web::Path<String>,
    payload: web::Json<UpdateFactStatusDTO>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let fact_id_str = path.into_inner();
    let fact_id = match FactID::parse_str(&fact_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid fact ID format"),
    };

    let ops = FactOps::new(context.get_ref());

    match ops.update_status(&fact_id, payload.into_inner(), None).await {
        Ok(fact) => HttpResponse::Ok().json(fact),
        Err(FactError::FactNotFound) => HttpResponse::NotFound().json("Fact not found"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn revoke_fact(
    path: web::Path<String>,
    context: web::Data<Arc<Context>>,
) -> impl Responder {
    let fact_id_str = path.into_inner();
    let fact_id = match FactID::parse_str(&fact_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid fact ID format"),
    };

    let ops = FactOps::new(context.get_ref());

    match ops.revoke_fact(&fact_id, None).await {
        Ok(fact) => HttpResponse::Ok().json(fact),
        Err(FactError::FactNotFound) => HttpResponse::NotFound().json("Fact not found"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
