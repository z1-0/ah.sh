use crate::paths::get_session_dir;
use crate::provider::{ProviderType, get_flake_contents};

use crate::session::types::{HISTORY_LIMIT, SESSION_ID_LEN, Session, SessionKey};
use anyhow::Result;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub fn generate_id(provider: ProviderType, languages: &[String]) -> String {
    let input = format!("{}:{}", provider, languages.join(","));
    let digest = blake3::hash(input.as_bytes());
    digest.to_hex().to_string()[..SESSION_ID_LEN].to_string()
}

pub fn list_sessions() -> Result<Vec<Session>> {
    let session_dir = get_session_dir()?;

    if !session_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("metadata.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path)?;
                if let Ok(session_meta) = serde_json::from_str::<Session>(&content) {
                    let modified_at = path
                        .metadata()
                        .and_then(|meta| meta.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);

                    sessions.push((
                        Session {
                            id: session_meta.id,
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

pub fn save_session(session: &Session) -> Result<()> {
    let session_dir = &session.get_dir()?;
    if !session_dir.exists() {
        std::fs::create_dir_all(session_dir)?;
    }

    let flake_contents = get_flake_contents(session.provider)(&session.languages)?;
    let flake_path = session_dir.join("flake.nix");
    std::fs::write(flake_path, flake_contents)?;

    let meta_path = session.get_dir()?.join("metadata.json");
    let content = serde_json::to_string_pretty(&session)?;
    std::fs::write(&meta_path, content)?;
    Ok(())
}

pub(crate) fn resolve_session(sessions: &[Session], key: &SessionKey) -> Result<Session> {
    match key {
        SessionKey::Index(idx) => {
            if *idx > 0 && *idx <= sessions.len() {
                Ok(sessions[idx - 1].clone())
            } else {
                anyhow::bail!("session '{}' not found", key)
            }
        }
        SessionKey::Id(id) => sessions
            .iter()
            .find(|s| s.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session '{}' not found", id)),
    }
}

pub fn find_session(key: &SessionKey) -> Result<Session> {
    let sessions = list_sessions()?;
    resolve_session(&sessions, key)
}

pub fn remove_session(session_id: &str) -> Result<bool> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(session_path)?;
    Ok(true)
}

pub fn clear_sessions() -> Result<usize> {
    let session_dir = get_session_dir()?;

    if !session_dir.exists() {
        return Ok(0);
    }

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

pub fn update_history(session: &Session, cwd: &Path) -> Result<()> {
    let session_dir = session.get_dir()?;
    let history_path = session_dir.join("history.json");

    // Read existing history
    let mut history: Vec<String> = if history_path.exists() {
        let content = fs::read_to_string(&history_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Remove existing entry for this path (if any)
    let cwd_str = cwd.to_string_lossy().into_owned();
    history.retain(|entry| *entry != cwd_str);

    // Add new entry at the beginning
    history.insert(0, cwd_str);

    // Keep only last HISTORY_LIMIT entries
    history.truncate(HISTORY_LIMIT);

    // Write back
    let content = serde_json::to_string_pretty(&history)?;
    fs::write(&history_path, content)?;

    Ok(())
}

pub fn find_by_path(path: &Path) -> Result<Vec<Session>> {
    let session_dir = get_session_dir()?;
    if !session_dir.exists() {
        return Ok(Vec::new());
    }

    let mut matching_sessions: Vec<(Session, SystemTime)> = Vec::new();
    let target_path = path.to_string_lossy().into_owned();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let session_path = entry.path();
        if session_path.is_dir() {
            let history_path = session_path.join("history.json");
            if history_path.exists()
                && let Ok(content) = fs::read_to_string(&history_path)
                && let Ok(history) = serde_json::from_str::<Vec<String>>(&content)
            {
                // Check if any entry matches the target path
                for history_entry in &history {
                    if *history_entry == target_path {
                        // Load session metadata
                        let meta_path = session_path.join("metadata.json");
                        if let Ok(session_content) = fs::read_to_string(&meta_path)
                            && let Ok(session) = serde_json::from_str::<Session>(&session_content)
                        {
                            // Get session dir mtime for sorting
                            let mtime = entry
                                .metadata()
                                .and_then(|m| m.modified())
                                .unwrap_or(SystemTime::UNIX_EPOCH);
                            matching_sessions.push((session, mtime));
                        }
                        break;
                    }
                }
            }
        }
    }

    // Sort by mtime descending (most recent first)
    matching_sessions.sort_by(|(_, a_mtime), (_, b_mtime)| b_mtime.cmp(a_mtime));

    Ok(matching_sessions.into_iter().map(|(s, _)| s).collect())
}
