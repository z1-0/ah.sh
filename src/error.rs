use crate::cmd::CommandError;
use crate::session::SessionError;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
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

    #[error(transparent)]
    Command(#[from] CommandError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error("{0}")]
    CliUsage(String),

    #[error("{0}")]
    Generic(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::CliUsage(_) => 2,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
