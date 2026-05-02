use sqlx::PgPool;

use crate::domain::{
    error::DomainError,
    post::{Post, UpdatePostRequest},
};

#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::PostNotFound)
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        sqlx::query_as::<_, Post>(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DomainError::PostNotFound,
            _ => DomainError::PostNotFound,
        })
    }

    pub async fn update_post(&self, id: i64, req: &UpdatePostRequest) -> Result<Post, DomainError> {
        sqlx::query_as::<_, Post>(
            r#"
            UPDATE posts
            SET title = $2, content = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.content)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DomainError::PostNotFound,
            _ => DomainError::PostNotFound,
        })
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|_| DomainError::PostNotFound)?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound);
        }

        Ok(())
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, DomainError> {
        sqlx::query_as::<_, Post>(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::PostNotFound)
    }

    pub async fn count_posts(&self) -> Result<i64, DomainError> {
        let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*)::BIGINT FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DomainError::PostNotFound)?;
        Ok(total)
    }
}
