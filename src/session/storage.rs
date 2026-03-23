use crate::paths::get_session_dir;
use crate::session::types::{SESSION_ID_LEN, Session, SessionKey};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
struct SessionMetadata {
    id: String,
    languages: Vec<String>,
    provider: String,
}

pub(crate) fn generate_id(provider: &str, languages: &[String]) -> String {
    let mut sorted_langs = languages.to_vec();
    sorted_langs.sort();

    let input = format!("{}:{}", provider, sorted_langs.join(","));
    let digest = blake3::hash(input.as_bytes());
    digest.to_hex().to_string()[..SESSION_ID_LEN].to_string()
}

pub(crate) fn list_sessions() -> Result<Vec<Session>> {
    let session_dir = get_session_dir()?;
    let mut sessions = Vec::new();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("metadata.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path)?;
                if let Ok(session_meta) = serde_json::from_str::<SessionMetadata>(&content) {
                    let modified_at = path
                        .metadata()
                        .and_then(|meta| meta.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);

                    sessions.push((
                        Session {
                            id: session_meta.id,
                            session_dir: path.clone(),
                            provider: session_meta.provider,
                            languages: session_meta.languages,
                        },
                        modified_at,
                    ));
                }
            }
        }
    }

    sessions.sort_by(|(a, a_mtime), (b, b_mtime)| match b_mtime.cmp(a_mtime) {
        Ordering::Equal => a.id.cmp(&b.id),
        other => other,
    });

    Ok(sessions.into_iter().map(|(session, _)| session).collect())
}

pub(crate) fn save_session(session: &Session) -> Result<()> {
    let session_path = get_session_dir()?.join(&session.id);
    if !session_path.exists() {
        fs::create_dir_all(&session_path)?;
    }
    let meta_path = session_path.join("metadata.json");
    let metadata = SessionMetadata {
        id: session.id.clone(),
        languages: session.languages.clone(),
        provider: session.provider.clone(),
    };
    let content = serde_json::to_string_pretty(&metadata)?;
    fs::write(&meta_path, content)?;
    Ok(())
}

pub(crate) fn resolve_session(sessions: &[Session], key: &SessionKey) -> Result<Session> {
    match key {
        SessionKey::Index(idx) => {
            if *idx > 0 && *idx <= sessions.len() {
                Ok(sessions[idx - 1].clone())
            } else {
                Err(anyhow::anyhow!("session '{}' not found", key))
            }
        }
        SessionKey::Id(id) => sessions
            .iter()
            .find(|s| s.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session '{}' not found", id)),
    }
}

pub(crate) fn find_session(key: &SessionKey) -> Result<Session> {
    let sessions = list_sessions()?;
    resolve_session(&sessions, key)
}

pub(crate) fn delete_session(session_id: &str) -> Result<bool> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(session_path)?;
    Ok(true)
}

pub(crate) fn clear_sessions() -> Result<usize> {
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
