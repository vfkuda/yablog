use async_trait::async_trait;
use tonic::Request;
use tonic::metadata::MetadataValue;

use crate::{AuthResponse, BlogClientError, BlogTransport, Post, PostsPage, User, blog};

pub struct GrpcBlogClient {
    client: blog::blog_service_client::BlogServiceClient<tonic::transport::Channel>,
}

impl GrpcBlogClient {
    pub async fn connect(addr: String) -> Result<Self, BlogClientError> {
        let client =
            blog::blog_service_client::BlogServiceClient::connect(normalize_grpc_addr(&addr))
                .await?;
        Ok(Self { client })
    }

    fn auth_from_grpc(response: blog::AuthResponse) -> Result<AuthResponse, BlogClientError> {
        let user = response
            .user
            .ok_or(BlogClientError::UnexpectedGrpcPayload)?;
        Ok(AuthResponse {
            token: response.token,
            user: User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at,
            },
        })
    }
}

#[async_trait]
impl BlogTransport for GrpcBlogClient {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .register(blog::RegisterRequest {
                username,
                email,
                password,
            })
            .await
            .map_err(map_grpc_status)?
            .into_inner();
        Self::auth_from_grpc(response)
    }

    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .login(blog::LoginRequest { username, password })
            .await
            .map_err(map_grpc_status)?
            .into_inner();
        Self::auth_from_grpc(response)
    }

    async fn create_post(
        &mut self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let mut req = Request::new(blog::CreatePostRequest { title, content });
        req.metadata_mut()
            .insert("authorization", bearer_metadata(token)?);
        let response = self
            .client
            .create_post(req)
            .await
            .map_err(map_grpc_status)?
            .into_inner();
        response.post.ok_or(BlogClientError::UnexpectedGrpcPayload)
    }

    async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get_post(blog::GetPostRequest { id })
            .await
            .map_err(map_grpc_status)?
            .into_inner();
        response.post.ok_or(BlogClientError::UnexpectedGrpcPayload)
    }

    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let mut req = Request::new(blog::UpdatePostRequest { id, title, content });
        req.metadata_mut()
            .insert("authorization", bearer_metadata(token)?);
        let response = self
            .client
            .update_post(req)
            .await
            .map_err(map_grpc_status)?
            .into_inner();
        response.post.ok_or(BlogClientError::UnexpectedGrpcPayload)
    }

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let mut req = Request::new(blog::DeletePostRequest { id });
        req.metadata_mut()
            .insert("authorization", bearer_metadata(token)?);
        self.client
            .delete_post(req)
            .await
            .map_err(map_grpc_status)?;
        Ok(())
    }

    async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<PostsPage, BlogClientError> {
        let response = self
            .client
            .list_posts(blog::ListPostsRequest { limit, offset })
            .await
            .map_err(map_grpc_status)?
            .into_inner();

        Ok(PostsPage {
            posts: response.posts,
            total: response.total,
            limit: response.limit,
            offset: response.offset,
        })
    }
}

fn normalize_grpc_addr(raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("http://{raw}")
    }
}

fn bearer_metadata(token: &str) -> Result<MetadataValue<tonic::metadata::Ascii>, BlogClientError> {
    MetadataValue::try_from(format!("Bearer {token}")).map_err(BlogClientError::InvalidMetadata)
}

fn map_grpc_status(status: tonic::Status) -> BlogClientError {
    match status.code() {
        tonic::Code::NotFound => BlogClientError::NotFound,
        tonic::Code::Unauthenticated => BlogClientError::Unauthorized,
        tonic::Code::PermissionDenied => BlogClientError::Forbidden,
        tonic::Code::InvalidArgument => {
            BlogClientError::InvalidRequest(status.message().to_string())
        }
        _ => BlogClientError::GrpcStatus(status),
    }
}
