use crate::error::{AppError, Result};
use crate::paths::{XdgDir, get_xdg_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

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
    let hash = blake3::hash(input.as_bytes());
    hash.to_hex().to_string()[..8].to_string()
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

pub fn find_session(input: &str) -> Result<Session> {
    let sessions = list_sessions()?;

    // Try numeric index (1-based)
    if let Ok(idx) = input.parse::<usize>()
        && idx > 0
        && idx <= sessions.len()
    {
        return Ok(sessions[idx - 1].clone());
    }

    // Try hash prefix
    for s in sessions {
        if s.id.starts_with(input) {
            return Ok(s);
        }
    }

    Err(AppError::Generic(format!("Session '{}' not found", input)))
}
