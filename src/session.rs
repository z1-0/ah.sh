use crate::error::{AppError, Result};
use crate::paths::{XdgDir, get_xdg_dir};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

pub const SESSION_ID_LEN: usize = 8;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid session key: {0}")]
    InvalidSelector(String),

    #[error("session '{0}' not found")]
    NotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub languages: Vec<String>,
    pub provider: String,
    pub created_at: u64,
}

impl Session {
    pub fn new(id: String, languages: Vec<String>, provider: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            languages,
            provider,
            created_at,
        }
    }
}

pub fn get_session_dir() -> Result<PathBuf> {
    let dir = get_xdg_dir(XdgDir::Cache)?.join("sessions");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn generate_id(provider: &str, languages: &[String]) -> String {
    let mut sorted_langs = languages.to_vec();
    sorted_langs.sort();

    let input = format!("{}:{}", provider, sorted_langs.join(","));
    let digest = blake3::hash(input.as_bytes());
    digest.to_hex().to_string()[..SESSION_ID_LEN].to_string()
}

pub fn list_sessions() -> Result<Vec<Session>> {
    let session_dir = get_session_dir()?;
    let mut sessions = Vec::new();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("metadata.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path)?;
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    sessions.push(session);
                }
            }
        }
    }

    // Sort by created_at descending (newest first)
    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sessions)
}

pub fn save_session(session: &Session) -> Result<()> {
    let session_path = get_session_dir()?.join(&session.id);
    if !session_path.exists() {
        fs::create_dir_all(&session_path)?;
    }
    let meta_path = session_path.join("metadata.json");
    let content = serde_json::to_string_pretty(session)?;
    fs::write(&meta_path, content)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum SessionKey {
    Index(usize),
    Id(String),
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionKey::Index(i) => write!(f, "{i}"),
            SessionKey::Id(id) => write!(f, "{id}"),
        }
    }
}

impl FromStr for SessionKey {
    type Err = AppError;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(SessionError::InvalidSelector(
                "session target cannot be empty".to_string(),
            )
            .into());
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input
                .parse::<usize>()
                .map_err(|_| SessionError::InvalidSelector("invalid session index".to_string()))?;
            if index == 0 {
                return Err(SessionError::InvalidSelector(
                    "session index must be greater than 0".to_string(),
                )
                .into());
            }
            return Ok(SessionKey::Index(index));
        }

        if !input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SessionError::InvalidSelector(
                "session id must contain only hexadecimal characters".to_string(),
            )
            .into());
        }

        if input.len() != SESSION_ID_LEN {
            return Err(SessionError::InvalidSelector(format!(
                "session id must be exactly {} hexadecimal characters",
                SESSION_ID_LEN
            ))
            .into());
        }

        Ok(SessionKey::Id(input.to_string()))
    }
}

pub fn resolve_session(sessions: &[Session], key: &SessionKey) -> Result<Session> {
    match key {
        SessionKey::Index(idx) => {
            if *idx > 0 && *idx <= sessions.len() {
                Ok(sessions[idx - 1].clone())
            } else {
                Err(SessionError::NotFound(key.to_string()).into())
            }
        }
        SessionKey::Id(id) => sessions
            .iter()
            .find(|s| s.id == *id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(id.clone()).into()),
    }
}

pub fn find_session(key: &SessionKey) -> Result<Session> {
    let sessions = list_sessions()?;
    resolve_session(&sessions, key)
}

pub fn delete_session(session_id: &str) -> Result<bool> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(session_path)?;
    Ok(true)
}

pub fn clear_sessions() -> Result<usize> {
    let session_dir = get_session_dir()?;
    let mut removed = 0usize;

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
            removed += 1;
        }
    }

    Ok(removed)
}
