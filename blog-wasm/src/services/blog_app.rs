use serde_json::json;
use wasm_bindgen::prelude::*;

use crate::{
    models::AuthResponse,
    services::{
        api::{
            DEFAULT_BASE_URL, create_post_request, delete_post_request, load_posts_request,
            login_request, register_request, update_post_request,
        },
        storage::{
            get_token_from_storage, remove_auth_from_storage, save_token_to_storage,
            save_user_id_to_storage,
        },
    },
};

#[wasm_bindgen]
pub struct BlogApp {
    server_url: String,
    jwt_token: Option<String>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BlogApp, JsValue> {
        Self::new_with_server_url(DEFAULT_BASE_URL.to_string())
    }

    pub fn new_with_server_url(server_url: String) -> Result<BlogApp, JsValue> {
        let token = get_token_from_storage()?;
        Ok(BlogApp {
            server_url,
            jwt_token: token,
        })
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let auth = register_request(&self.server_url, username, email, password)
            .await
            .map_err(to_js_error)?;

        self.jwt_token = Some(auth.token.clone());
        self.save_auth_to_storage_internal(&auth)?;
        serde_wasm_bindgen::to_value(&auth).map_err(to_js_error)
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        let auth = login_request(&self.server_url, username, password)
            .await
            .map_err(to_js_error)?;

        self.jwt_token = Some(auth.token.clone());
        self.save_auth_to_storage_internal(&auth)?;
        serde_wasm_bindgen::to_value(&auth).map_err(to_js_error)
    }

    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let posts = load_posts_request(&self.server_url)
            .await
            .map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&posts).map_err(to_js_error)
    }

    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let post = create_post_request(&self.server_url, &self.require_token()?, title, content)
            .await
            .map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&post).map_err(to_js_error)
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        let post =
            update_post_request(&self.server_url, &self.require_token()?, id, title, content)
                .await
                .map_err(to_js_error)?;

        serde_wasm_bindgen::to_value(&post).map_err(to_js_error)
    }

    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        delete_post_request(&self.server_url, &self.require_token()?, id)
            .await
            .map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&json!({ "ok": true })).map_err(to_js_error)
    }

    pub fn is_authenticated(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.jwt_token.is_some()).map_err(to_js_error)
    }

    pub fn save_token_to_storage(&self, token: String) -> Result<JsValue, JsValue> {
        save_token_to_storage(&token)?;
        serde_wasm_bindgen::to_value(&json!({ "ok": true })).map_err(to_js_error)
    }

    pub fn get_token_from_storage(&self) -> Result<JsValue, JsValue> {
        let token = get_token_from_storage()?;
        serde_wasm_bindgen::to_value(&token).map_err(to_js_error)
    }

    pub fn logout(&mut self) -> Result<JsValue, JsValue> {
        self.jwt_token = None;
        remove_auth_from_storage()?;
        serde_wasm_bindgen::to_value(&json!({ "ok": true })).map_err(to_js_error)
    }

    pub fn server_url(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.server_url).map_err(to_js_error)
    }
}

impl BlogApp {
    fn save_auth_to_storage_internal(&self, auth: &AuthResponse) -> Result<(), JsValue> {
        save_token_to_storage(&auth.token)?;
        save_user_id_to_storage(auth.user.id)?;
        Ok(())
    }

    fn require_token(&self) -> Result<String, JsValue> {
        self.jwt_token
            .clone()
            .ok_or_else(|| js_error("missing JWT token"))
    }
}

fn to_js_error(err: impl core::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn js_error(msg: impl Into<String>) -> JsValue {
    JsValue::from_str(&msg.into())
}
