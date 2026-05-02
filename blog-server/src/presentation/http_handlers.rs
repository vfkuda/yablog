use std::sync::Arc;

use actix_web::{HttpResponse, Responder, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    application::auth_service::AuthService,
    application::{
        auth_service::AuthServiceError,
        blog_service::{BlogService, BlogServiceError},
    },
    domain::{
        error::DomainError,
        post::{CreatePostRequest, Post, UpdatePostRequest},
        user::{LoginRequest, RegistrationRequest, User},
    },
    presentation::{
        middleware::AuthenticatedUser,
        observability::{RequestOutcome, log_request_handled},
    },
};

const HTTP_PROTOCOL: &str = "http";
const DEFAULT_POSTS_LIMIT: i64 = 10;
const MIN_POSTS_LIMIT: i64 = 1;
const MAX_POSTS_LIMIT: i64 = 100;
const DEFAULT_POSTS_OFFSET: i64 = 0;

#[derive(Debug, Serialize)]
struct PublicUser {
    id: i64,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: PublicUser,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PostsListResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    // публичные маршруты для ауторизации отдельно от защищенных
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login)),
    );
}

pub async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    _blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<RegistrationRequest>,
) -> impl Responder {
    match auth_service.register(payload.into_inner()).await {
        Ok((token, user)) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "register",
                "201 Created",
                RequestOutcome::Success,
                None,
            );

            HttpResponse::Created().json(AuthResponse {
                token,
                user: to_public_user(user),
            })
        }
        Err(err) => map_auth_error_to_http("register", err),
    }
}

pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    _blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    match auth_service.login(payload.into_inner()).await {
        Ok((token, user)) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "login",
                "200 OK",
                RequestOutcome::Success,
                None,
            );

            HttpResponse::Ok().json(AuthResponse {
                token,
                user: to_public_user(user),
            })
        }
        Err(err) => map_auth_error_to_http("login", err),
    }
}

fn to_public_user(user: User) -> PublicUser {
    PublicUser {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at,
    }
}

pub async fn create_post(
    auth_user: AuthenticatedUser,
    blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<CreatePostRequest>,
) -> impl Responder {
    match blog_service
        .create_post(auth_user.user_id, payload.into_inner())
        .await
    {
        Ok(post) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "create_post",
                "201 Created",
                RequestOutcome::Success,
                None,
            );
            HttpResponse::Created().json(post)
        }
        Err(err) => map_blog_error_to_http("create_post", err),
    }
}

pub async fn get_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> impl Responder {
    match blog_service.get_post(path.into_inner()).await {
        Ok(post) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "get_post",
                "200 OK",
                RequestOutcome::Success,
                None,
            );
            HttpResponse::Ok().json(post)
        }
        Err(err) => map_blog_error_to_http("get_post", err),
    }
}

pub async fn update_post(
    auth_user: AuthenticatedUser,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    payload: web::Json<UpdatePostRequest>,
) -> impl Responder {
    match blog_service
        .update_post(path.into_inner(), auth_user.user_id, payload.into_inner())
        .await
    {
        Ok(post) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "update_post",
                "200 OK",
                RequestOutcome::Success,
                None,
            );
            HttpResponse::Ok().json(post)
        }
        Err(err) => map_blog_error_to_http("update_post", err),
    }
}

pub async fn delete_post(
    auth_user: AuthenticatedUser,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> impl Responder {
    match blog_service
        .delete_post(path.into_inner(), auth_user.user_id)
        .await
    {
        Ok(()) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "delete_post",
                "204 No Content",
                RequestOutcome::Success,
                None,
            );
            HttpResponse::NoContent().finish()
        }
        Err(err) => map_blog_error_to_http("delete_post", err),
    }
}

pub async fn list_posts(
    blog_service: web::Data<Arc<BlogService>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    // контроль паарметров
    let limit = query
        .limit
        .unwrap_or(DEFAULT_POSTS_LIMIT)
        .clamp(MIN_POSTS_LIMIT, MAX_POSTS_LIMIT);
    let offset = query
        .offset
        .unwrap_or(DEFAULT_POSTS_OFFSET)
        .max(DEFAULT_POSTS_OFFSET);

    match blog_service.list_posts(limit, offset).await {
        Ok((posts, total)) => {
            log_request_handled::<str>(
                HTTP_PROTOCOL,
                "list_posts",
                "200 OK",
                RequestOutcome::Success,
                None,
            );
            HttpResponse::Ok().json(PostsListResponse {
                posts,
                total,
                limit,
                offset,
            })
        }
        Err(err) => map_blog_error_to_http("list_posts", err),
    }
}

fn map_auth_error_to_http(operation: &'static str, err: AuthServiceError) -> HttpResponse {
    match err {
        AuthServiceError::Domain(DomainError::UserAlreadyExists) => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "409 Conflict",
                RequestOutcome::ClientError,
                Some(&err),
            );
            HttpResponse::Conflict().body("user already exists")
        }
        AuthServiceError::Domain(DomainError::UserNotFound)
        | AuthServiceError::Domain(DomainError::InvalidCredentials) => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "401 Unauthorized",
                RequestOutcome::ClientError,
                Some(&err),
            );
            HttpResponse::Unauthorized().body("invalid credentials")
        }
        _ => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "500 Internal Server Error",
                RequestOutcome::ServerError,
                Some(&err),
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn map_blog_error_to_http(operation: &'static str, err: BlogServiceError) -> HttpResponse {
    match err {
        BlogServiceError::Domain(DomainError::PostNotFound) => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "404 Not Found",
                RequestOutcome::ClientError,
                Some(&err),
            );
            HttpResponse::NotFound().finish()
        }
        BlogServiceError::Domain(DomainError::Forbidden) => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "403 Forbidden",
                RequestOutcome::ClientError,
                Some(&err),
            );
            HttpResponse::Forbidden().finish()
        }
        _ => {
            log_request_handled(
                HTTP_PROTOCOL,
                operation,
                "500 Internal Server Error",
                RequestOutcome::ServerError,
                Some(&err),
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
