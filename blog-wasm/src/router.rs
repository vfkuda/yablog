use dioxus::prelude::*;

use crate::pages::home::HomePage;

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        HomePage {}
    }
}
