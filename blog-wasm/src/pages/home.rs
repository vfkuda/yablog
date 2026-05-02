use dioxus::prelude::*;

use crate::{
    BlogApp,
    components::status_bar::StatusBar,
    models::{AuthResponse, Post},
    services::{
        api::{DEFAULT_BASE_URL, load_posts_request},
        storage::{get_token_from_storage, get_user_id_from_storage},
    },
    state::{StatusKind, StatusMessage},
    styles::POST_TEXTAREA_ROWS,
};

#[component]
pub fn HomePage() -> Element {
    let posts = use_signal(Vec::<Post>::new);
    let mut status = use_signal(StatusMessage::default);
    let mut is_loading = use_signal(|| false);
    let mut current_user_id = use_signal(|| get_user_id_from_storage().ok().flatten());
    let mut is_authenticated = use_signal(|| get_token_from_storage().ok().flatten().is_some());
    let mut is_login_mode = use_signal(|| true);
    let mut did_init = use_signal(|| false);

    let mut reg_username = use_signal(String::new);
    let mut reg_email = use_signal(String::new);
    let mut reg_password = use_signal(String::new);

    let mut login_username = use_signal(String::new);
    let mut login_password = use_signal(String::new);

    let mut post_title = use_signal(String::new);
    let mut post_content = use_signal(String::new);

    let mut editing_post_id = use_signal(|| None::<i64>);
    let mut edit_title = use_signal(String::new);
    let mut edit_content = use_signal(String::new);

    use_effect(move || {
        if did_init() {
            return;
        }

        did_init.set(true);
        to_owned![posts, status, is_loading];
        spawn(async move {
            refresh_posts(posts, status, is_loading).await;
        });
    });

    let post_items = posts.read().clone();
    let auth_text = if is_authenticated() {
        "залогинен"
    } else {
        "не залогинен"
    };
    let auth_class = if is_authenticated() { "ok" } else { "no" };
    let status_view = status.read().clone();

    rsx! {
        div { class: "page",
            h1 { "YaBlog Dioxus" }

            section { class: "card",
                h2 { "Регистрация и вход" }
                p {
                    "Статус: "
                    span { class: "{auth_class}", "{auth_text}" }
                }
                p { class: "muted", "Сервер: {DEFAULT_BASE_URL}" }

                if !is_authenticated() {
                    div { class: "toolbar",
                        button {
                            class: if is_login_mode() { "secondary" } else { "ghost" },
                            onclick: move |_| is_login_mode.set(true),
                            "Вход"
                        }
                        button {
                            class: if !is_login_mode() { "secondary" } else { "ghost" },
                            onclick: move |_| is_login_mode.set(false),
                            "Регистрация"
                        }
                    }

                    if is_login_mode() {
                        div {
                            p { class: "muted", "Введите username и password." }
                            input {
                                value: "{login_username}",
                                placeholder: "username",
                                oninput: move |event| login_username.set(event.value())
                            }
                            input {
                                value: "{login_password}",
                                r#type: "password",
                                placeholder: "password",
                                oninput: move |event| login_password.set(event.value())
                            }
                            button {
                                disabled: is_loading(),
                                onclick: move |_| {
                                    let username = login_username().trim().to_string();
                                    let password = login_password().trim().to_string();
                                    if username.is_empty() || password.is_empty() {
                                        status.set(StatusMessage::error("Заполните username и password."));
                                        return;
                                    }

                                    is_loading.set(true);
                                    to_owned![
                                        posts,
                                        status,
                                        is_loading,
                                        is_authenticated,
                                        current_user_id,
                                        login_username,
                                        login_password
                                    ];
                                    spawn(async move {
                                        let mut app = match BlogApp::new() {
                                            Ok(app) => app,
                                            Err(err) => {
                                                status.set(StatusMessage::error(
                                                    &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                                ));
                                                is_loading.set(false);
                                                return;
                                            }
                                        };

                                        match app.login(username, password).await {
                                            Ok(value) => {
                                                match serde_wasm_bindgen::from_value::<AuthResponse>(value) {
                                                    Ok(auth) => {
                                                        is_authenticated.set(true);
                                                        current_user_id.set(Some(auth.user.id));
                                                        login_username.set(String::new());
                                                        login_password.set(String::new());
                                                        status.set(StatusMessage::success("Вход выполнен."));
                                                        refresh_posts(posts, status, is_loading).await;
                                                    }
                                                    Err(err) => {
                                                        status.set(StatusMessage::error(&err.to_string()));
                                                        is_loading.set(false);
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                status.set(StatusMessage::error(
                                                    &err.as_string().unwrap_or_else(|| "Ошибка входа.".to_string())
                                                ));
                                                is_loading.set(false);
                                            }
                                        }
                                    });
                                },
                                "Войти"
                            }
                        }
                    } else {
                        div {
                            p { class: "muted", "Создайте нового пользователя." }
                            input {
                                value: "{reg_username}",
                                placeholder: "username",
                                oninput: move |event| reg_username.set(event.value())
                            }
                            input {
                                value: "{reg_email}",
                                placeholder: "email",
                                oninput: move |event| reg_email.set(event.value())
                            }
                            input {
                                value: "{reg_password}",
                                r#type: "password",
                                placeholder: "password",
                                oninput: move |event| reg_password.set(event.value())
                            }
                            button {
                                disabled: is_loading(),
                                onclick: move |_| {
                                    let username = reg_username().trim().to_string();
                                    let email = reg_email().trim().to_string();
                                    let password = reg_password().trim().to_string();
                                    if username.is_empty() || email.is_empty() || password.is_empty() {
                                        status.set(StatusMessage::error("Заполните username, email и password."));
                                        return;
                                    }

                                    is_loading.set(true);
                                    to_owned![
                                        posts,
                                        status,
                                        is_loading,
                                        is_authenticated,
                                        current_user_id,
                                        reg_username,
                                        reg_email,
                                        reg_password
                                    ];
                                    spawn(async move {
                                        let mut app = match BlogApp::new() {
                                            Ok(app) => app,
                                            Err(err) => {
                                                status.set(StatusMessage::error(
                                                    &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                                ));
                                                is_loading.set(false);
                                                return;
                                            }
                                        };

                                        match app.register(username, email, password).await {
                                            Ok(value) => {
                                                match serde_wasm_bindgen::from_value::<AuthResponse>(value) {
                                                    Ok(auth) => {
                                                        is_authenticated.set(true);
                                                        current_user_id.set(Some(auth.user.id));
                                                        reg_username.set(String::new());
                                                        reg_email.set(String::new());
                                                        reg_password.set(String::new());
                                                        status.set(StatusMessage::success("Регистрация выполнена."));
                                                        refresh_posts(posts, status, is_loading).await;
                                                    }
                                                    Err(err) => {
                                                        status.set(StatusMessage::error(&err.to_string()));
                                                        is_loading.set(false);
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                status.set(StatusMessage::error(
                                                    &err.as_string().unwrap_or_else(|| "Ошибка регистрации.".to_string())
                                                ));
                                                is_loading.set(false);
                                            }
                                        }
                                    });
                                },
                                "Зарегистрироваться"
                            }
                        }
                    }
                } else {
                    div {
                        p { class: "muted", "Токен взят из браузера. Для смены пользователя выйдите из аккаунта." }
                        button {
                            class: "danger",
                            disabled: is_loading(),
                            onclick: move |_| {
                                match BlogApp::new() {
                                    Ok(mut app) => {
                                        if let Err(err) = app.logout() {
                                            status.set(StatusMessage::error(
                                                &err.as_string().unwrap_or_else(|| "Не удалось выйти.".to_string())
                                            ));
                                            return;
                                        }
                                    }
                                    Err(err) => {
                                        status.set(StatusMessage::error(
                                            &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                        ));
                                        return;
                                    }
                                }

                                is_authenticated.set(false);
                                current_user_id.set(None);
                                editing_post_id.set(None);
                                edit_title.set(String::new());
                                edit_content.set(String::new());
                                status.set(StatusMessage::success("Вы вышли из аккаунта."));
                            },
                            "Выйти"
                        }
                    }
                }
            }

            StatusBar { status: status_view }

            if is_authenticated() {
                section { class: "card",
                    h2 { "Новый пост" }
                    input {
                        value: "{post_title}",
                        placeholder: "Заголовок",
                        oninput: move |event| post_title.set(event.value())
                    }
                    textarea {
                        value: "{post_content}",
                        rows: "{POST_TEXTAREA_ROWS}",
                        placeholder: "Содержание",
                        oninput: move |event| post_content.set(event.value())
                    }
                    button {
                        disabled: is_loading(),
                        onclick: move |_| {
                            let title = post_title().trim().to_string();
                            let content = post_content().trim().to_string();
                            if title.is_empty() || content.is_empty() {
                                status.set(StatusMessage::error("Заполните заголовок и содержание поста."));
                                return;
                            }

                            is_loading.set(true);
                            to_owned![posts, status, is_loading, post_title, post_content];
                            spawn(async move {
                                let app = match BlogApp::new() {
                                    Ok(app) => app,
                                    Err(err) => {
                                        status.set(StatusMessage::error(
                                            &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                        ));
                                        is_loading.set(false);
                                        return;
                                    }
                                };

                                match app.create_post(title, content).await {
                                    Ok(_) => {
                                        post_title.set(String::new());
                                        post_content.set(String::new());
                                        status.set(StatusMessage::success("Пост создан."));
                                        refresh_posts(posts, status, is_loading).await;
                                    }
                                    Err(err) => {
                                        status.set(StatusMessage::error(
                                            &err.as_string().unwrap_or_else(|| "Ошибка создания поста.".to_string())
                                        ));
                                        is_loading.set(false);
                                    }
                                }
                            });
                        },
                        "Опубликовать"
                    }
                }
            }

            section { class: "card",
                h2 { "Посты" }
                div { class: "toolbar",
                    button {
                        class: "secondary",
                        disabled: is_loading(),
                        onclick: move |_| {
                            is_loading.set(true);
                            to_owned![posts, status, is_loading];
                            spawn(async move {
                                refresh_posts(posts, status, is_loading).await;
                            });
                        },
                        "Обновить список"
                    }
                }

                if post_items.is_empty() {
                    div { class: "empty", "Пока постов нет." }
                } else {
                    for post in post_items {
                        {
                            let can_edit = current_user_id() == Some(post.author_id);
                            let is_open = editing_post_id() == Some(post.id);
                            let post_id = post.id;
                            let post_title_text = post.title.clone();
                            let post_content_text = post.content.clone();
                            rsx! {
                                article {
                                    key: "{post.id}",
                                    class: if can_edit { "post clickable" } else { "post" },
                                    onclick: move |_| {
                                        if can_edit {
                                            editing_post_id.set(Some(post_id));
                                            edit_title.set(post_title_text.clone());
                                            edit_content.set(post_content_text.clone());
                                        }
                                    },
                                    h3 { "{post.title}" }
                                    p { class: "muted", "ID: {post.id} | author_id: {post.author_id}" }
                                    p { "{post.content}" }
                                    p { class: "muted", "Создан: {post.created_at}" }
                                    p { class: "muted", "Обновлён: {post.updated_at}" }

                                    if can_edit {
                                        div { class: "toolbar",
                                            button {
                                                class: "ghost",
                                                onclick: move |_| {
                                                    editing_post_id.set(Some(post.id));
                                                    edit_title.set(post.title.clone());
                                                    edit_content.set(post.content.clone());
                                                },
                                                "Редактировать"
                                            }
                                            button {
                                                class: "danger",
                                                disabled: is_loading(),
                                                onclick: move |_| {
                                                    is_loading.set(true);
                                                    editing_post_id.set(None);
                                                    to_owned![posts, status, is_loading];
                                                    spawn(async move {
                                                        let app = match BlogApp::new() {
                                                            Ok(app) => app,
                                                            Err(err) => {
                                                                status.set(StatusMessage::error(
                                                                    &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                                                ));
                                                                is_loading.set(false);
                                                                return;
                                                            }
                                                        };

                                                        match app.delete_post(post.id).await {
                                                            Ok(_) => {
                                                                status.set(StatusMessage::success("Пост удалён."));
                                                                refresh_posts(posts, status, is_loading).await;
                                                            }
                                                            Err(err) => {
                                                                status.set(StatusMessage::error(
                                                                    &err.as_string().unwrap_or_else(|| "Ошибка удаления поста.".to_string())
                                                                ));
                                                                is_loading.set(false);
                                                            }
                                                        }
                                                    });
                                                },
                                                "Удалить"
                                            }
                                        }

                                        if is_open {
                                            div {
                                                class: "post-editor",
                                                onclick: move |event| event.stop_propagation(),
                                                h3 { "Редактирование" }
                                                input {
                                                    value: "{edit_title}",
                                                    placeholder: "Заголовок",
                                                    oninput: move |event| edit_title.set(event.value())
                                                }
                                                textarea {
                                                    value: "{edit_content}",
                                                    rows: "{POST_TEXTAREA_ROWS}",
                                                    placeholder: "Содержание",
                                                    oninput: move |event| edit_content.set(event.value())
                                                }
                                                div { class: "toolbar",
                                                    button {
                                                        disabled: is_loading(),
                                                        onclick: move |_| {
                                                            let title = edit_title().trim().to_string();
                                                            let content = edit_content().trim().to_string();
                                                            if title.is_empty() || content.is_empty() {
                                                                status.set(StatusMessage::error("Заполните заголовок и содержание перед сохранением."));
                                                                return;
                                                            }

                                                            is_loading.set(true);
                                                            editing_post_id.set(None);
                                                            to_owned![posts, status, is_loading];
                                                            spawn(async move {
                                                                let app = match BlogApp::new() {
                                                                    Ok(app) => app,
                                                                    Err(err) => {
                                                                        status.set(StatusMessage::error(
                                                                            &err.as_string().unwrap_or_else(|| "Не удалось создать приложение.".to_string())
                                                                        ));
                                                                        is_loading.set(false);
                                                                        return;
                                                                    }
                                                                };

                                                                match app.update_post(post.id, title, content).await {
                                                                    Ok(_) => {
                                                                        status.set(StatusMessage::success("Пост обновлён."));
                                                                        refresh_posts(posts, status, is_loading).await;
                                                                    }
                                                                    Err(err) => {
                                                                        status.set(StatusMessage::error(
                                                                            &err.as_string().unwrap_or_else(|| "Ошибка обновления поста.".to_string())
                                                                        ));
                                                                        is_loading.set(false);
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        "Сохранить"
                                                    }
                                                    button {
                                                        class: "ghost",
                                                        onclick: move |_| {
                                                            editing_post_id.set(None);
                                                            edit_title.set(String::new());
                                                            edit_content.set(String::new());
                                                        },
                                                        "Отмена"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn refresh_posts(
    mut posts: Signal<Vec<Post>>,
    mut status: Signal<StatusMessage>,
    mut is_loading: Signal<bool>,
) {
    match load_posts_request(DEFAULT_BASE_URL).await {
        Ok(response) => {
            posts.set(response.posts);
            if status().kind != StatusKind::Success {
                status.set(StatusMessage::default());
            }
        }
        Err(_) => status.set(StatusMessage::error("сервер не доступен")),
    }

    is_loading.set(false);
}
