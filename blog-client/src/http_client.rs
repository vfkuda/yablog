use async_trait::async_trait;

use crate::{AuthResponse, BlogClientError, BlogTransport, Post, PostsPage};

pub struct HttpBlogClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpBlogClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn parse_http_json<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, BlogClientError> {
        if response.status().is_success() {
            Ok(response.json::<T>().await?)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(map_http_error(status, body))
        }
    }
}

#[async_trait]
impl BlogTransport for HttpBlogClient {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let url = format!("{}/api/auth/register", self.base_url);
        let resp = self
            .client
            .post(url)
            .json(&RegisterHttpRequest {
                username,
                email,
                password,
            })
            .send()
            .await?;
        Self::parse_http_json(resp).await
    }

    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let url = format!("{}/api/auth/login", self.base_url);
        let resp = self
            .client
            .post(url)
            .json(&LoginHttpRequest { username, password })
            .send()
            .await?;
        Self::parse_http_json(resp).await
    }

    async fn create_post(
        &mut self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts", self.base_url);
        let resp = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&CreatePostHttpRequest { title, content })
            .send()
            .await?;
        let post: HttpPost = Self::parse_http_json(resp).await?;
        Ok(post.into_proto())
    }

    async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let resp = self.client.get(url).send().await?;
        let post: HttpPost = Self::parse_http_json(resp).await?;
        Ok(post.into_proto())
    }

    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let resp = self
            .client
            .put(url)
            .bearer_auth(token)
            .json(&UpdatePostHttpRequest { title, content })
            .send()
            .await?;
        let post: HttpPost = Self::parse_http_json(resp).await?;
        Ok(post.into_proto())
    }

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let resp = self.client.delete(url).bearer_auth(token).send().await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(map_http_error(status, body))
        }
    }

    async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<PostsPage, BlogClientError> {
        let url = format!(
            "{}/api/posts?limit={}&offset={}",
            self.base_url, limit, offset
        );
        let resp = self.client.get(url).send().await?;
        let page: HttpPostsListResponse = Self::parse_http_json(resp).await?;
        Ok(page.into_posts_page())
    }
}

fn map_http_error(status: reqwest::StatusCode, body: String) -> BlogClientError {
    match status {
        reqwest::StatusCode::NOT_FOUND => BlogClientError::NotFound,
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            BlogClientError::Unauthorized
        }
        reqwest::StatusCode::BAD_REQUEST => BlogClientError::InvalidRequest(body),
        _ => BlogClientError::HttpApi { status, body },
    }
}

#[derive(Debug, serde::Serialize)]
struct RegisterHttpRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, serde::Serialize)]
struct LoginHttpRequest {
    username: String,
    password: String,
}

#[derive(Debug, serde::Serialize)]
struct CreatePostHttpRequest {
    title: String,
    content: String,
}

#[derive(Debug, serde::Serialize)]
struct UpdatePostHttpRequest {
    title: String,
    content: String,
}

#[derive(Debug, serde::Deserialize)]
struct HttpPost {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: String,
    updated_at: String,
}

impl HttpPost {
    fn into_proto(self) -> Post {
        Post {
            id: self.id,
            title: self.title,
            content: self.content,
            author_id: self.author_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct HttpPostsListResponse {
    posts: Vec<HttpPost>,
    total: i64,
    limit: i64,
    offset: i64,
}

impl HttpPostsListResponse {
    fn into_posts_page(self) -> PostsPage {
        PostsPage {
            posts: self.posts.into_iter().map(HttpPost::into_proto).collect(),
            total: self.total,
            limit: self.limit as i32,
            offset: self.offset as i32,
        }
    }
}
