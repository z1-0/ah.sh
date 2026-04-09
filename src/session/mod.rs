mod storage;
mod types;

use crate::provider::{Language, ProviderType, to_supported_languages};
use anyhow::Result;
use std::collections::HashSet;

pub use storage::*;
pub use types::*;

pub fn generate_id(provider: ProviderType, languages: &[String]) -> String {
    let input = format!("{}:{}", provider, languages.join(","));
    let digest = blake3::hash(input.as_bytes());
    digest.to_hex().to_string()[..SESSION_ID_LEN].to_string()
}

pub fn find_by_key(key: &SessionKey) -> Result<Session> {
    match key {
        SessionKey::Id(id) => {
            // Direct lookup: O(1) instead of loading all sessions O(n)
            let session_path = crate::paths::get_session_dir()?.join(id);
            if !session_path.exists() {
                anyhow::bail!("session '{}' not found", id);
            }
            let meta_path = session_path.join("metadata.json");
            let content = std::fs::read_to_string(&meta_path)?;
            let session = serde_json::from_str(&content)?;
            Ok(session)
        }
        SessionKey::Index(idx) => {
            // Index requires loading all sessions
            let sessions = list_sessions()?;
            if *idx > 0 && *idx <= sessions.len() {
                Ok(sessions[idx - 1].clone())
            } else {
                anyhow::bail!("session '{}' not found", key)
            }
        }
    }
}

pub fn find_session(provider: ProviderType, languages: &[Language]) -> Result<Option<Session>> {
    let supported_languages = to_supported_languages(provider, languages)?;
    let session_id = generate_id(provider, &supported_languages);

    // Direct lookup: O(1) instead of loading all sessions O(n)
    let session_path = crate::paths::get_session_dir()?.join(&session_id);
    if !session_path.exists() {
        return Ok(None);
    }

    let meta_path = session_path.join("metadata.json");
    let content = std::fs::read_to_string(&meta_path)?;
    let session = serde_json::from_str(&content)?;
    Ok(Some(session))
}

pub fn remove_sessions(keys: &[SessionKey]) -> Result<Option<SessionRemoveResult>> {
    if keys.is_empty() {
        return Ok(None);
    }

    let sessions = list_sessions()?;
    if sessions.is_empty() {
        return Ok(None);
    }

    // Build index for O(1) lookup instead of O(n) per key
    let sessions_by_id: std::collections::HashMap<_, _> =
        sessions.iter().map(|s| (s.id.as_str(), s)).collect();

    let mut removed_ids = Vec::new();
    let mut missing_keys = Vec::new();
    let mut deduped_session_ids = HashSet::new();

    for key in keys {
        let session: Option<Session> = match key {
            SessionKey::Id(id) => sessions_by_id.get(id.as_str()).map(|s| (*s).clone()),
            SessionKey::Index(idx) => {
                if *idx > 0 && *idx <= sessions.len() {
                    Some(sessions[idx - 1].clone())
                } else {
                    None
                }
            }
        };

        if let Some(session) = session {
            if deduped_session_ids.insert(session.id.clone()) {
                let session_id = session.id.clone();
                if remove_session(&session_id)? {
                    removed_ids.push(session_id);
                } else {
                    missing_keys.push(session_id);
                }
            }
        } else {
            missing_keys.push(key.to_string());
        }
    }

    Ok(Some(SessionRemoveResult {
        removed_ids,
        missing_keys,
    }))
}

pub fn create_session(provider: ProviderType, languages: Vec<Language>) -> Result<Session> {
    let supported_languages = to_supported_languages(provider, &languages)?;

    let session_id = generate_id(provider, &supported_languages);

    let session = Session {
        id: session_id,
        provider,
        languages: supported_languages,
    };

    save_session(&session)?;

    Ok(session)
}
