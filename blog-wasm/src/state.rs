#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

impl StatusKind {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Info => "status info",
            Self::Success => "status success",
            Self::Error => "status error",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
}

impl StatusMessage {
    pub fn success(text: &str) -> Self {
        Self {
            text: text.to_string(),
            kind: StatusKind::Success,
        }
    }

    pub fn error(text: &str) -> Self {
        Self {
            text: text.to_string(),
            kind: StatusKind::Error,
        }
    }
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            text: "Готово к работе.".to_string(),
            kind: StatusKind::Info,
        }
    }
}
