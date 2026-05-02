mod grpc_client;
mod http_client;
pub mod error;

pub mod blog {
    tonic::include_proto!("blog");
}

use async_trait::async_trait;
use grpc_client::GrpcBlogClient;
use http_client::HttpBlogClient;

pub use error::BlogClientError;
pub type Post = blog::Post;

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone)]
pub struct PostsPage {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[async_trait]
pub trait BlogTransport: Send {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError>;

    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError>;

    async fn create_post(
        &mut self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;

    async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError>;

    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError>;

    async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<PostsPage, BlogClientError>;
}

pub struct BlogClient {
    transport: Box<dyn BlogTransport>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let transport: Box<dyn BlogTransport> = match transport {
            Transport::Http(base_url) => Box::new(HttpBlogClient::new(base_url)),
            Transport::Grpc(addr) => Box::new(GrpcBlogClient::connect(addr).await?),
        };

        Ok(Self {
            transport,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<AuthResponse, BlogClientError> {
        let auth = self
            .transport
            .register(username.into(), email.into(), password.into())
            .await?;
        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub async fn login(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<AuthResponse, BlogClientError> {
        let auth = self
            .transport
            .login(username.into(), password.into())
            .await?;
        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub async fn create_post(
        &mut self,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::MissingToken)?;
        self.transport
            .create_post(token, title.into(), content.into())
            .await
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        self.transport.get_post(id).await
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::MissingToken)?;
        self.transport
            .update_post(token, id, title.into(), content.into())
            .await
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::MissingToken)?;
        self.transport.delete_post(token, id).await
    }

    pub async fn list_posts(
        &mut self,
        limit: i32,
        offset: i32,
    ) -> Result<PostsPage, BlogClientError> {
        self.transport.list_posts(limit, offset).await
    }
}
