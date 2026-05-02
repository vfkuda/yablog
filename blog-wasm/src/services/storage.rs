use wasm_bindgen::JsValue;
use web_sys::window;

pub const TOKEN_STORAGE_KEY: &str = "blog_token";
pub const USER_ID_STORAGE_KEY: &str = "blog_user_id";

pub fn save_token_to_storage(token: &str) -> Result<(), JsValue> {
    let storage = browser_storage()?;
    storage.set_item(TOKEN_STORAGE_KEY, token)?;
    Ok(())
}

pub fn get_token_from_storage() -> Result<Option<String>, JsValue> {
    let storage = browser_storage()?;
    storage.get_item(TOKEN_STORAGE_KEY)
}

pub fn save_user_id_to_storage(user_id: i64) -> Result<(), JsValue> {
    let storage = browser_storage()?;
    storage.set_item(USER_ID_STORAGE_KEY, &user_id.to_string())?;
    Ok(())
}

pub fn get_user_id_from_storage() -> Result<Option<i64>, JsValue> {
    let storage = browser_storage()?;
    let value = storage.get_item(USER_ID_STORAGE_KEY)?;

    match value {
        Some(raw) => raw
            .parse::<i64>()
            .map(Some)
            .map_err(|_| js_error("cannot parse saved user id")),
        None => Ok(None),
    }
}

pub fn remove_auth_from_storage() -> Result<(), JsValue> {
    let storage = browser_storage()?;
    storage.remove_item(TOKEN_STORAGE_KEY)?;
    storage.remove_item(USER_ID_STORAGE_KEY)?;
    Ok(())
}

fn browser_storage() -> Result<web_sys::Storage, JsValue> {
    window()
        .ok_or_else(|| js_error("window is not available"))?
        .local_storage()?
        .ok_or_else(|| js_error("localStorage is not available"))
}

fn js_error(msg: impl Into<String>) -> JsValue {
    JsValue::from_str(&msg.into())
}
