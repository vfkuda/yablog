mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use std::{path::PathBuf, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use tonic::transport::Server;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const HTTP_SERVER_ADDR: &str = "0.0.0.0";
const HTTP_SERVER_PORT: u16 = 8080;
const GRPC_SERVER_ADDR: &str = "0.0.0.0";
const GRPC_SERVER_PORT: u16 = 50051;
const HOURS_PER_DAY: i64 = 24;
const MINUTES_PER_HOUR: i64 = 60;
const SECONDS_PER_MINUTE: i64 = 60;
const DEFAULT_JWT_TTL_SECONDS: i64 = HOURS_PER_DAY * MINUTES_PER_HOUR * SECONDS_PER_MINUTE;
const CORS_PREFLIGHT_CACHE_SECONDS: usize = 3600;

fn load_dot_env() {
    // берем локальный .env рядом с крейтом, потом общий fallback
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    if dotenvy::from_path(&env_path).is_err() {
        dotenvy::dotenv().ok();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dot_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("loading parameters");
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set. Example: postgres://user:pass@localhost/yablog")?;

    let jwt_secret =
        std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is not set. Example: super-secret")?;
    let jwt_ttl_seconds = std::env::var("JWT_TTL_SECONDS")
        .ok()
        .and_then(|raw| raw.parse::<i64>().ok())
        .unwrap_or(DEFAULT_JWT_TTL_SECONDS);
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();

    info!("creating pool");
    let pool = infrastructure::database::create_pool(&database_url)
        .await
        .map_err(|e| format!("Database connection error: {e}"))?;

    info!("run migrations");
    infrastructure::database::run_migrations(&pool)
        .await
        .map_err(|e| format!("Database migrating error: {e}"))?;

    use data::post_repository::PostRepository as PostgresPostRepository;
    let post_repository = Arc::new(PostgresPostRepository::new(pool.clone()));

    use data::user_repository::UserRepository as PostgresUserRepository;
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));

    let jwt_service = Arc::new(infrastructure::jwt::JwtService::new(
        &jwt_secret,
        jwt_ttl_seconds,
    ));
    let auth_service = Arc::new(application::auth_service::AuthService::new(
        user_repository,
        jwt_service.clone(),
    ));
    let blog_service = Arc::new(application::blog_service::BlogService::new(post_repository));

    let http_auth_service = auth_service.clone();
    let http_blog_service = blog_service.clone();
    let http_jwt_service = jwt_service.clone();
    let http_app_env = app_env.clone();
    let http_cors_allowed_origins = cors_allowed_origins.clone();

    info!("running HTTP server");
    let http_server = HttpServer::new(move || {
        let cors = build_cors(&http_app_env, &http_cors_allowed_origins);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(http_auth_service.clone()))
            .app_data(web::Data::new(http_blog_service.clone()))
            .app_data(web::Data::new(http_jwt_service.clone()))
            .configure(presentation::http_handlers::configure_auth_routes)
            .service(
                web::scope("/api/posts")
                    .route("", web::get().to(presentation::http_handlers::list_posts))
                    .route(
                        "/{id}",
                        web::get().to(presentation::http_handlers::get_post),
                    )
                    .service(
                        web::scope("")
                            .wrap(HttpAuthentication::bearer(
                                presentation::middleware::jwt_validator,
                            ))
                            .route("", web::post().to(presentation::http_handlers::create_post))
                            .route(
                                "/{id}",
                                web::put().to(presentation::http_handlers::update_post),
                            )
                            .route(
                                "/{id}",
                                web::delete().to(presentation::http_handlers::delete_post),
                            ),
                    ),
            )
    })
    .bind((HTTP_SERVER_ADDR, HTTP_SERVER_PORT))?
    .run();
    info!(
        "HTTP server listening on {}:{}",
        HTTP_SERVER_ADDR, HTTP_SERVER_PORT
    );

    info!("running gRPC server");
    let grpc_addr = format!("{GRPC_SERVER_ADDR}:{GRPC_SERVER_PORT}").parse()?;
    let grpc_service = presentation::grpc_service::BlogGrpcService::new(
        auth_service.clone(),
        blog_service.clone(),
        jwt_service.clone(),
    );
    let grpc_server = Server::builder()
        .add_service(
            presentation::grpc_service::pb::blog_service_server::BlogServiceServer::new(
                grpc_service,
            ),
        )
        .serve(grpc_addr);
    info!(
        "gRPC server listening on  {}:{}",
        GRPC_SERVER_ADDR, GRPC_SERVER_PORT
    );

    tokio::select! {
        http_res = http_server => {
            match http_res {
                Ok(()) => {
                    warn!("HTTP server stopped");
                    Ok(())
                }
                Err(err) => {
                    error!("HTTP server failed: {err}");
                    Err(err.into())
                }
            }
        }
        grpc_res = grpc_server => {
            match grpc_res {
                Ok(()) => {
                    warn!("gRPC server stopped");
                    Ok(())
                }
                Err(err) => {
                    error!("gRPC server failed: {err}");
                    Err(err.into())
                }
            }
        }
    }
}

fn build_cors(app_env: &str, cors_allowed_origins: &str) -> Cors {
    let base = Cors::default()
        .allow_any_header()
        .allowed_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .max_age(CORS_PREFLIGHT_CACHE_SECONDS);

    if app_env.eq_ignore_ascii_case("production") {
        // в проде режем origin'ы явно, чтобы фронтенд не получил лишний доступ
        let origins: Vec<String> = cors_allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        origins
            .into_iter()
            .fold(base, |cors, origin| cors.allowed_origin(&origin))
    } else {
        base.allow_any_origin()
    }
}
