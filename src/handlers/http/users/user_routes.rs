use std::sync::Arc;

use actix_web::{
    web,
    web::{Json},
    HttpRequest,
    HttpResponse,
    Responder,
};


use crate::{
    context::Context,
    core::{domain::user::user_type::NewUser, operation::user_ops::UserOps },
};


pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(web::scope("/api/users")
        .route("/newuser", web::post().to(new_user)).service(web::scope("")
        .route("/all", web::get().to(load_users))
        // .route("/username/{una}", web::get().to(load_users_username))
        // .route("/userid/{id}", web::get().to(load_users_id))
    ));
}

async fn new_user(context: web::Data<Arc<Context>>, payload: Json<NewUser>) -> impl Responder
{
    let user_repo =  context.get_ref().get_user_repo();
    let auth_repo=   context.get_ref().get_auth_repo();
    let perm_repo=  context.get_ref().get_perm_repo();
    
    let user_ops = UserOps::new(&user_repo, perm_repo.as_ref(), auth_repo.as_ref(), &context).await;

    match user_ops.create_user(payload.into_inner(), true).await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_users(req: HttpRequest, context: web::Data<Arc<Context>>) -> impl Responder
{
    let user_repo =  context.get_ref().get_user_repo();
    let auth_repo=   context.get_ref().get_auth_repo();
    let perm_repo=  context.get_ref().get_perm_repo();

    let user_ops = UserOps::new(&user_repo, perm_repo.as_ref(), auth_repo.as_ref(), &context).await;

    match user_ops.load_users(req).await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
// 
// async fn load_users_username(path: Path<String>, context: web::Data<Arc<Context>>) -> impl Responder
// {
//     let user_service = UserService { user_repo:  context.get_ref().get_user_repo(),
//                                      auth_repo:  context.get_ref().get_auth_repo(),
//                                      perms_repo: context.get_ref().get_perms_repo(), };
// 
//     match user_service.load_user_by_username(path.into_inner())
//                       .await
//     {
//         Ok(user) => HttpResponse::Ok().json(user),
//         Err(err) => HttpResponse::InternalServerError().json(err.message),
//     }
// }
// 
// async fn load_users_id(path: Path<ObjectId>, context: web::Data<Arc<Context>>) -> impl Responder
// {
//     let user_service = UserService { user_repo:  context.get_ref().get_user_repo(),
//                                      auth_repo:  context.get_ref().get_auth_repo(),
//                                      perms_repo: context.get_ref().get_perms_repo(), };
// 
//     match user_service.load_user_by_id(path.into_inner())
//                       .await
//     {
//         Ok(user) => HttpResponse::Ok().json(user),
//         Err(err) => HttpResponse::InternalServerError().json(err.message),
//     }
// }
