use sqlx::{PgPool, postgres::PgPoolOptions};

const DEFAULT_MAX_DB_CONNECTIONS: u32 = 5;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_DB_CONNECTIONS)
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    // запускаем миграции на старте
    sqlx::migrate!("./migrations").run(pool).await
}
