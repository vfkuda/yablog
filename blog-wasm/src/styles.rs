const APP_MAX_WIDTH_PX: usize = 860;
const APP_PADDING_PX: usize = 16;
const CARD_RADIUS_PX: usize = 6;
const CONTROL_MARGIN_TOP_PX: usize = 8;
const CONTROL_PADDING_PX: usize = 8;
pub const POST_TEXTAREA_ROWS: usize = 5;

const APP_STYLE: &str = r#"
    :root {
        color-scheme: light;
        --bg: #f4f4f4;
        --card: #ffffff;
        --text: #222222;
        --muted: #666666;
        --border: #cfcfcf;
        --primary: #2f6fed;
        --danger: #b42318;
        --success: #166534;
    }
    * {
        box-sizing: border-box;
    }
    body {
        margin: 0;
        font-family: sans-serif;
        color: var(--text);
        background: var(--bg);
    }
    .page {
        max-width: __APP_MAX_WIDTH_PX__px;
        margin: 0 auto;
        padding: __APP_PADDING_PX__px;
    }
    .card {
        background: var(--card);
        border: 1px solid var(--border);
        border-radius: __CARD_RADIUS_PX__px;
        padding: __APP_PADDING_PX__px;
        margin-bottom: 12px;
    }
    h1, h2, h3, p {
        margin-top: 0;
    }
    input, textarea, button {
        width: 100%;
        margin-top: __CONTROL_MARGIN_TOP_PX__px;
        padding: __CONTROL_PADDING_PX__px;
        border-radius: 4px;
        border: 1px solid var(--border);
        font: inherit;
        background: #fff;
    }
    button {
        cursor: pointer;
        background: var(--primary);
        color: #fff;
        border: none;
    }
    button.secondary {
        background: #555555;
    }
    button.ghost {
        color: var(--text);
        background: #ececec;
        border: 1px solid var(--border);
    }
    button.danger {
        background: var(--danger);
    }
    button:disabled {
        opacity: 0.65;
        cursor: not-allowed;
    }
    .toolbar {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }
    .toolbar button {
        width: auto;
    }
    .status {
        min-height: 44px;
    }
    .status.info {
        background: #eef4ff;
        border-color: #b6c7ef;
    }
    .status.success {
        background: #eefbf1;
        border-color: #b7d7bf;
        color: var(--success);
    }
    .status.error {
        background: #fff3f1;
        border-color: #efc2bd;
        color: var(--danger);
    }
    .muted {
        color: var(--muted);
    }
    .post {
        border: 1px solid var(--border);
        border-radius: 4px;
        padding: 12px;
        margin-top: 10px;
        background: #fff;
    }
    .post.clickable {
        cursor: pointer;
    }
    .post h3 {
        margin-bottom: 6px;
    }
    .post-editor {
        border-top: 1px solid var(--border);
        margin-top: 12px;
        padding-top: 12px;
    }
    .ok {
        color: var(--success);
        font-weight: 600;
    }
    .no {
        color: var(--danger);
        font-weight: 600;
    }
    .empty {
        padding: 16px;
        text-align: center;
        color: var(--muted);
    }
"#;

pub fn app_style() -> String {
    APP_STYLE
        .replace("__APP_MAX_WIDTH_PX__", &APP_MAX_WIDTH_PX.to_string())
        .replace("__APP_PADDING_PX__", &APP_PADDING_PX.to_string())
        .replace("__CARD_RADIUS_PX__", &CARD_RADIUS_PX.to_string())
        .replace(
            "__CONTROL_MARGIN_TOP_PX__",
            &CONTROL_MARGIN_TOP_PX.to_string(),
        )
        .replace("__CONTROL_PADDING_PX__", &CONTROL_PADDING_PX.to_string())
}
