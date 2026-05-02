use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    application::{
        auth_service::{AuthService, AuthServiceError},
        blog_service::{BlogService as AppBlogService, BlogServiceError},
    },
    domain::{
        error::DomainError,
        post::{
            CreatePostRequest as DomainCreatePostRequest,
            UpdatePostRequest as DomainUpdatePostRequest,
        },
        user::{
            LoginRequest as DomainLoginRequest, RegistrationRequest as DomainRegistrationRequest,
        },
    },
    infrastructure::jwt::JwtService,
    presentation::observability::{RequestOutcome, log_request_handled},
};

const GRPC_PROTOCOL: &str = "grpc";
const DEFAULT_POSTS_LIMIT: i32 = 10;
const MAX_POSTS_LIMIT: i32 = 100;
const MIN_POST_ID: i64 = 1;
const MIN_GPRC_LIST_OFFSET: i32 = 0;

pub mod pb {
    tonic::include_proto!("blog");
}

#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<AppBlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<AppBlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }
}

#[tonic::async_trait]
impl pb::blog_service_server::BlogService for BlogGrpcService {
    async fn register(
        &self,
        request: Request<pb::RegisterRequest>,
    ) -> Result<Response<pb::AuthResponse>, Status> {
        let req = request.into_inner();
        if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
            return Err(log_grpc_client_status(
                "register",
                Status::invalid_argument("username, email and password are required"),
            ));
        }

        let (token, user) = self
            .auth_service
            .register(DomainRegistrationRequest {
                username: req.username,
                email: req.email,
                password: req.password,
            })
            .await
            .map_err(|err| map_auth_error("register", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "register",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::AuthResponse {
            token,
            user: Some(to_proto_user(user)),
        }))
    }

    async fn login(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> Result<Response<pb::AuthResponse>, Status> {
        let req = request.into_inner();
        if req.username.is_empty() || req.password.is_empty() {
            return Err(log_grpc_client_status(
                "login",
                Status::invalid_argument("username and password are required"),
            ));
        }

        let (token, user) = self
            .auth_service
            .login(DomainLoginRequest {
                username: req.username,
                password: req.password,
            })
            .await
            .map_err(|err| map_auth_error("login", err))?;

        log_request_handled::<str>(GRPC_PROTOCOL, "login", "OK", RequestOutcome::Success, None);

        Ok(Response::new(pb::AuthResponse {
            token,
            user: Some(to_proto_user(user)),
        }))
    }

    async fn create_post(
        &self,
        request: Request<pb::CreatePostRequest>,
    ) -> Result<Response<pb::PostResponse>, Status> {
        let claims = authenticate_request("create_post", &self.jwt_service, &request)?;

        let req = request.into_inner();
        if req.title.is_empty() || req.content.is_empty() {
            return Err(log_grpc_client_status(
                "create_post",
                Status::invalid_argument("title and content are required"),
            ));
        }

        let post = self
            .blog_service
            .create_post(
                claims.user_id,
                DomainCreatePostRequest {
                    title: req.title,
                    content: req.content,
                },
            )
            .await
            .map_err(|err| map_blog_error("create_post", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "create_post",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::PostResponse {
            post: Some(to_proto_post(post)),
        }))
    }

    async fn get_post(
        &self,
        request: Request<pb::GetPostRequest>,
    ) -> Result<Response<pb::PostResponse>, Status> {
        let req = request.into_inner();

        if req.id < MIN_POST_ID {
            return Err(log_grpc_client_status(
                "get_post",
                Status::invalid_argument("id must be greater than 0"),
            ));
        }

        let post = self
            .blog_service
            .get_post(req.id)
            .await
            .map_err(|err| map_blog_error("get_post", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "get_post",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::PostResponse {
            post: Some(to_proto_post(post)),
        }))
    }

    async fn update_post(
        &self,
        request: Request<pb::UpdatePostRequest>,
    ) -> Result<Response<pb::PostResponse>, Status> {
        let claims = authenticate_request("update_post", &self.jwt_service, &request)?;

        let req = request.into_inner();

        if req.id < MIN_POST_ID {
            return Err(log_grpc_client_status(
                "update_post",
                Status::invalid_argument("id must be greater than 0"),
            ));
        }
        if req.title.is_empty() || req.content.is_empty() {
            return Err(log_grpc_client_status(
                "update_post",
                Status::invalid_argument("title and content are required"),
            ));
        }

        let post = self
            .blog_service
            .update_post(
                req.id,
                claims.user_id,
                DomainUpdatePostRequest {
                    title: req.title,
                    content: req.content,
                },
            )
            .await
            .map_err(|err| map_blog_error("update_post", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "update_post",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::PostResponse {
            post: Some(to_proto_post(post)),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<pb::DeletePostRequest>,
    ) -> Result<Response<pb::DeletePostResponse>, Status> {
        let claims = authenticate_request("delete_post", &self.jwt_service, &request)?;

        let req = request.into_inner();

        if req.id < MIN_POST_ID {
            return Err(log_grpc_client_status(
                "delete_post",
                Status::invalid_argument("id must be greater than 0"),
            ));
        }

        self.blog_service
            .delete_post(req.id, claims.user_id)
            .await
            .map_err(|err| map_blog_error("delete_post", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "delete_post",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::DeletePostResponse {}))
    }

    async fn list_posts(
        &self,
        request: Request<pb::ListPostsRequest>,
    ) -> Result<Response<pb::ListPostsResponse>, Status> {
        let req = request.into_inner();
        // контроль корректных limit/offset
        let limit = if req.limit <= MIN_GPRC_LIST_OFFSET {
            DEFAULT_POSTS_LIMIT
        } else {
            req.limit.min(MAX_POSTS_LIMIT)
        } as i64;
        let offset = req.offset.max(MIN_GPRC_LIST_OFFSET) as i64;

        let (posts, total) = self
            .blog_service
            .list_posts(limit, offset)
            .await
            .map_err(|err| map_blog_error("list_posts", err))?;

        log_request_handled::<str>(
            GRPC_PROTOCOL,
            "list_posts",
            "OK",
            RequestOutcome::Success,
            None,
        );

        Ok(Response::new(pb::ListPostsResponse {
            posts: posts.into_iter().map(to_proto_post).collect(),
            total,
            limit: limit as i32,
            offset: offset as i32,
        }))
    }
}

fn authenticate_request<T>(
    operation: &'static str,
    jwt_service: &JwtService,
    request: &Request<T>,
) -> Result<crate::infrastructure::jwt::Claims, Status> {
    // достаем  токен вручную из metadata
    let raw_auth = request.metadata().get("authorization").ok_or_else(|| {
        log_grpc_client_status(
            operation,
            Status::unauthenticated("missing authorization metadata"),
        )
    })?;
    let raw_auth = raw_auth.to_str().map_err(|_| {
        log_grpc_client_status(
            operation,
            Status::unauthenticated("invalid authorization metadata"),
        )
    })?;
    let token = raw_auth.strip_prefix("Bearer ").ok_or_else(|| {
        log_grpc_client_status(
            operation,
            Status::unauthenticated("authorization must be Bearer token"),
        )
    })?;

    jwt_service
        .verify_token(token)
        .map_err(|_| log_grpc_client_status(operation, Status::unauthenticated("invalid token")))
}

fn map_auth_error(operation: &'static str, err: AuthServiceError) -> Status {
    match err {
        AuthServiceError::Domain(DomainError::UserAlreadyExists) => log_grpc_client_error(
            operation,
            Status::already_exists("user already exists"),
            &err,
        ),
        AuthServiceError::Domain(DomainError::UserNotFound)
        | AuthServiceError::Domain(DomainError::InvalidCredentials) => log_grpc_client_error(
            operation,
            Status::unauthenticated("invalid credentials"),
            &err,
        ),
        AuthServiceError::PasswordHash => log_grpc_client_error(
            operation,
            Status::invalid_argument("invalid password"),
            &err,
        ),
        _ => log_grpc_server_error(operation, Status::internal("internal server error"), &err),
    }
}

fn map_blog_error(operation: &'static str, err: BlogServiceError) -> Status {
    match err {
        BlogServiceError::Domain(DomainError::PostNotFound) => {
            log_grpc_client_error(operation, Status::not_found("post not found"), &err)
        }
        BlogServiceError::Domain(DomainError::Forbidden) => log_grpc_client_error(
            operation,
            Status::permission_denied("permission denied"),
            &err,
        ),
        _ => log_grpc_server_error(operation, Status::internal("internal server error"), &err),
    }
}

fn log_grpc_client_status(operation: &'static str, status: Status) -> Status {
    log_request_handled(
        GRPC_PROTOCOL,
        operation,
        grpc_code_name(status.code()),
        RequestOutcome::ClientError,
        Some(&status),
    );
    status
}

fn log_grpc_client_error<E>(operation: &'static str, status: Status, err: &E) -> Status
where
    E: std::fmt::Display + ?Sized,
{
    log_request_handled(
        GRPC_PROTOCOL,
        operation,
        grpc_code_name(status.code()),
        RequestOutcome::ClientError,
        Some(err),
    );
    status
}

fn log_grpc_server_error<E>(operation: &'static str, status: Status, err: &E) -> Status
where
    E: std::fmt::Display + ?Sized,
{
    log_request_handled(
        GRPC_PROTOCOL,
        operation,
        grpc_code_name(status.code()),
        RequestOutcome::ServerError,
        Some(err),
    );
    status
}

fn grpc_code_name(code: tonic::Code) -> &'static str {
    match code {
        tonic::Code::Ok => "OK",
        tonic::Code::Cancelled => "CANCELLED",
        tonic::Code::Unknown => "UNKNOWN",
        tonic::Code::InvalidArgument => "INVALID_ARGUMENT",
        tonic::Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
        tonic::Code::NotFound => "NOT_FOUND",
        tonic::Code::AlreadyExists => "ALREADY_EXISTS",
        tonic::Code::PermissionDenied => "PERMISSION_DENIED",
        tonic::Code::ResourceExhausted => "RESOURCE_EXHAUSTED",
        tonic::Code::FailedPrecondition => "FAILED_PRECONDITION",
        tonic::Code::Aborted => "ABORTED",
        tonic::Code::OutOfRange => "OUT_OF_RANGE",
        tonic::Code::Unimplemented => "UNIMPLEMENTED",
        tonic::Code::Internal => "INTERNAL",
        tonic::Code::Unavailable => "UNAVAILABLE",
        tonic::Code::DataLoss => "DATA_LOSS",
        tonic::Code::Unauthenticated => "UNAUTHENTICATED",
    }
}

fn to_proto_user(user: crate::domain::user::User) -> pb::User {
    pb::User {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at.to_rfc3339(),
    }
}

fn to_proto_post(post: crate::domain::post::Post) -> pb::Post {
    pb::Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: post.created_at.to_rfc3339(),
        updated_at: post.updated_at.to_rfc3339(),
    }
}
