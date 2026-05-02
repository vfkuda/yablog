use dioxus::prelude::*;

use crate::state::StatusMessage;

#[component]
pub fn StatusBar(status: StatusMessage) -> Element {
    rsx! {
        section { class: "card {status.kind.class_name()}",
            "{status.text}"
        }
    }
}
