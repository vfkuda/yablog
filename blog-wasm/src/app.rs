use dioxus::prelude::*;

use crate::{router::AppRouter, styles::app_style};

pub fn app() -> Element {
    rsx! {
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" }
        style { "{app_style()}" }
        AppRouter {}
    }
}
