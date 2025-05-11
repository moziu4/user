use std::{
    env,
    pin::Pin,
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{ServiceRequest, ServiceResponse},
    Error,
    HttpResponse,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::core::domain::auth::auth_type::Claims;

pub struct RoleGuard
{
    pub required_permission: u32,
}

impl RoleGuard
{
    pub fn new(required_permission: u32) -> Self
    {
        Self { required_permission }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleGuard
    where S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
          S::Future: 'static,
          B: 'static
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RoleGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future
    {
        ok(RoleGuardMiddleware { service,
                                 required_permission: self.required_permission })
    }
}

pub struct RoleGuardMiddleware<S>
{
    service:             S,
    required_permission: u32,
}

impl<S, B> Service<ServiceRequest> for RoleGuardMiddleware<S>
    where S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
          S::Future: 'static,
          B: 'static
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future
    {
        let auth_header = req.headers().get("Authorization");

        if auth_header.is_none()
        {
            let response = HttpResponse::Unauthorized().finish()
                                                       .map_into_right_body();
            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        let auth_header = auth_header.unwrap().to_str().unwrap_or_default();
        if !auth_header.starts_with("Bearer ")
        {
            let response = HttpResponse::Unauthorized().finish()
                                                       .map_into_right_body();
            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        let token = &auth_header[7..]; // Remover "Bearer "

        let secret = env::var("SECRET_KEY").unwrap().to_string();
        let validation = Validation::default();
        let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_ref()), &validation);

        if token_data.is_err()
        {
            let response = HttpResponse::Unauthorized().finish()
                                                       .map_into_right_body();
            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        let claims = token_data.unwrap().claims;
        if !claims.permissions
                  .contains(&self.required_permission)
        {
            let response = HttpResponse::Forbidden().finish()
                                                    .map_into_right_body();
            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
