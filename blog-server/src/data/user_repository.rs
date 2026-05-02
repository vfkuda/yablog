use sqlx::PgPool;

use crate::domain::{error::DomainError, user::User};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                DomainError::UserAlreadyExists
            }
            _ => DomainError::UserNotFound,
        })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DomainError::UserNotFound,
            _ => DomainError::UserNotFound,
        })
    }
}
