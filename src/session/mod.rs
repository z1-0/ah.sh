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

pub fn find_session(provider: ProviderType, languages: &[Language]) -> Result<Option<Session>> {
    let supported_languages = to_supported_languages(provider, languages)?;
    let session_id = generate_id(provider, &supported_languages);
    try_session_by_id(&session_id)
}

pub fn remove_sessions(keys: &[SessionKey]) -> Result<Option<SessionRemoveResult>> {
    if keys.is_empty() {
        return Ok(None);
    }

    let mut removed_ids = Vec::new();
    let mut missing_keys = Vec::new();
    let mut deduped_session_ids = HashSet::new();

    for key in keys {
        let session = try_session_by_key(key)?;

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
