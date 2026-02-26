use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AhError {
    #[error("Environment variable not set: {0}")]
    EnvVarNotFound(#[from] std::env::VarError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Unsupported languages: {0:?}")]
    UnsupportedLanguages(Vec<String>),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("{0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, AhError>;
