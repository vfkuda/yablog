use std::sync::Arc;

use thiserror::Error;

use crate::{
    data::post_repository::PostRepository,
    domain::{
        error::DomainError,
        post::{CreatePostRequest, Post, UpdatePostRequest},
    },
};

#[derive(Debug, Error)]
pub enum BlogServiceError {
    #[error(transparent)]
    Domain(#[from] DomainError),
}

#[derive(Clone)]
pub struct BlogService {
    post_repository: Arc<PostRepository>,
}

impl BlogService {
    pub fn new(post_repository: Arc<PostRepository>) -> Self {
        Self { post_repository }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        req: CreatePostRequest,
    ) -> Result<Post, BlogServiceError> {
        self.post_repository
            .create_post(&req.title, &req.content, author_id)
            .await
            .map_err(BlogServiceError::from)
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogServiceError> {
        self.post_repository
            .get_post(id)
            .await
            .map_err(BlogServiceError::from)
    }

    pub async fn update_post(
        &self,
        id: i64,
        user_id: i64,
        req: UpdatePostRequest,
    ) -> Result<Post, BlogServiceError> {
        let existing = self.post_repository.get_post(id).await?;
        if existing.author_id != user_id {
            return Err(BlogServiceError::Domain(DomainError::Forbidden));
        }

        self.post_repository
            .update_post(id, &req)
            .await
            .map_err(BlogServiceError::from)
    }

    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), BlogServiceError> {
        let existing = self.post_repository.get_post(id).await?;
        if existing.author_id != user_id {
            return Err(BlogServiceError::Domain(DomainError::Forbidden));
        }
        self.post_repository
            .delete_post(id)
            .await
            .map_err(BlogServiceError::from)
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), BlogServiceError> {
        let posts = self.post_repository.list_posts(limit, offset).await?;
        let total = self.post_repository.count_posts().await?;
        Ok((posts, total))
    }
}
