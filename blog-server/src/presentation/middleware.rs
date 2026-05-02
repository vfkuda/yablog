use std::{
    future::{Ready, ready},
    sync::Arc,
};

use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest,
    dev::{Payload, ServiceRequest},
    error::ErrorUnauthorized,
    web,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::{
    infrastructure::jwt::JwtService,
    presentation::observability::{RequestOutcome, log_request_handled},
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    // pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req
        .app_data::<web::Data<Arc<JwtService>>>()
        .map(|data| data.get_ref().clone())
    {
        Some(service) => service,
        None => {
            log_request_handled::<str>(
                "http",
                "jwt_validator",
                "401 Unauthorized",
                RequestOutcome::ServerError,
                None,
            );
            return Err((ErrorUnauthorized("jwt service not configured"), req));
        }
    };

    let claims = match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => claims,
        Err(err) => {
            log_request_handled(
                "http",
                "jwt_validator",
                "401 Unauthorized",
                RequestOutcome::ClientError,
                Some(&err),
            );
            return Err((ErrorUnauthorized("invalid token"), req));
        }
    };

    req.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.user_id,
        // username: claims.username,
    });

    Ok(req)
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => ready(Err(ErrorUnauthorized("unauthorized"))),
        }
    }
}
