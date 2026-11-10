use std::{env, future::Future, pin::Pin};

use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use perms::{verify_context, RequestContext};

pub struct VerifiedContext(pub RequestContext);

impl FromRequest for VerifiedContext {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::to_owned);

        Box::pin(async move {
            let token = token.ok_or_else(|| ErrorUnauthorized("Falta el contexto firmado"))?;
            let public_key = env::var("PERMS_PUBLIC_KEY")
                .map_err(|_| ErrorUnauthorized("PERMS_PUBLIC_KEY no configurada"))?;
            let issuer = env::var("PERMS_ISSUER")
                .map_err(|_| ErrorUnauthorized("PERMS_ISSUER no configurado"))?;
            let audience = env::var("PERMS_AUDIENCE")
                .map_err(|_| ErrorUnauthorized("PERMS_AUDIENCE no configurado"))?;
            let key_id = env::var("PERMS_KEY_ID")
                .map_err(|_| ErrorUnauthorized("PERMS_KEY_ID no configurado"))?;

            let context = verify_context(
                &token,
                public_key.as_bytes(),
                &issuer,
                &audience,
                &key_id,
            )
            .map_err(|_| ErrorUnauthorized("Contexto firmado inválido o expirado"))?;

            Ok(Self(context))
        })
    }
}