use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::json;

use crate::models::{AuthResponse, Post, PostsListResponse};

pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

pub async fn register_request(
    server_url: &str,
    username: String,
    email: String,
    password: String,
) -> Result<AuthResponse, String> {
    let url = format!("{server_url}/api/auth/register");
    let body = json!({
        "username": username,
        "email": email,
        "password": password,
    });

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(map_request_error)?
        .send()
        .await
        .map_err(map_request_error)?;

    parse_json_response(response).await
}

pub async fn login_request(
    server_url: &str,
    username: String,
    password: String,
) -> Result<AuthResponse, String> {
    let url = format!("{server_url}/api/auth/login");
    let body = json!({
        "username": username,
        "password": password,
    });

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(map_request_error)?
        .send()
        .await
        .map_err(map_request_error)?;

    parse_json_response(response).await
}

pub async fn load_posts_request(server_url: &str) -> Result<PostsListResponse, String> {
    let url = format!("{server_url}/api/posts");
    let response = Request::get(&url).send().await.map_err(map_request_error)?;

    parse_json_response(response).await
}

pub async fn create_post_request(
    server_url: &str,
    token: &str,
    title: String,
    content: String,
) -> Result<Post, String> {
    let url = format!("{server_url}/api/posts");
    let body = json!({
        "title": title,
        "content": content,
    });

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .body(body.to_string())
        .map_err(map_request_error)?
        .send()
        .await
        .map_err(map_request_error)?;

    parse_json_response(response).await
}

pub async fn update_post_request(
    server_url: &str,
    token: &str,
    id: i64,
    title: String,
    content: String,
) -> Result<Post, String> {
    let url = format!("{server_url}/api/posts/{id}");
    let body = json!({
        "title": title,
        "content": content,
    });

    let response = Request::put(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .body(body.to_string())
        .map_err(map_request_error)?
        .send()
        .await
        .map_err(map_request_error)?;

    parse_json_response(response).await
}

pub async fn delete_post_request(server_url: &str, token: &str, id: i64) -> Result<(), String> {
    let url = format!("{server_url}/api/posts/{id}");
    let response = Request::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(map_request_error)?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(map_http_error(status, &body))
    }
}

async fn parse_json_response<T>(response: gloo_net::http::Response) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status, &body));
    }

    response.json().await.map_err(|err| err.to_string())
}

fn map_request_error(err: gloo_net::Error) -> String {
    let message = err.to_string();
    if message.contains("Failed to fetch") {
        "сервер не доступен".to_string()
    } else {
        message
    }
}

fn map_http_error(status: u16, body: &str) -> String {
    match status {
        400 => "Некорректный запрос.".to_string(),
        401 => "Неверный логин или пароль.".to_string(),
        403 => "Недостаточно прав для этого действия.".to_string(),
        404 => "Запрошенный ресурс не найден.".to_string(),
        409 => {
            if body.contains("user already exists") {
                "Пользователь уже существует.".to_string()
            } else {
                "Конфликт данных.".to_string()
            }
        }
        500..=599 => "Ошибка сервера.".to_string(),
        _ => {
            if body.is_empty() {
                format!("Ошибка HTTP: {status}.")
            } else {
                format!("Ошибка HTTP: {status}. {body}")
            }
        }
    }
}
