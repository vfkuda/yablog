mod app;
mod components;
mod models;
mod pages;
mod router;
mod services;
mod state;
mod styles;

pub use services::blog_app::BlogApp;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    dioxus::launch(app::app);
}
