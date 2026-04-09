mod storage;
mod types;

use crate::provider::{Language, ProviderType, to_supported_languages};
use anyhow::Result;
use std::collections::HashSet;

pub use storage::*;
pub use types::*;

pub fn find_session(provider: ProviderType, languages: &[Language]) -> Result<Option<Session>> {
    let supported_languages = to_supported_languages(provider, languages)?;

    let session_id = storage::generate_id(provider, &supported_languages);
    let session = storage::find_session(&SessionKey::Id(session_id))?;
    Ok(Some(session))
}

pub fn resolve_session_dir(key: &SessionKey) -> Result<Session> {
    storage::find_session(key)
}

pub fn remove_sessions(keys: &[SessionKey]) -> Result<Option<SessionRemoveResult>> {
    if keys.is_empty() {
        return Ok(None);
    }

    let sessions = storage::list_sessions()?;
    if sessions.is_empty() {
        return Ok(None);
    }

    let mut removed_ids = Vec::new();
    let mut missing_keys = Vec::new();
    let mut deduped_session_ids = HashSet::new();

    for key in keys {
        match storage::find_session(key) {
            Ok(session) => {
                if deduped_session_ids.insert(session.id.clone()) {
                    let session_id = session.id.clone();
                    if storage::remove_session(&session_id)? {
                        removed_ids.push(session_id);
                    } else {
                        missing_keys.push(session_id);
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to resolve session: {}", e);
                missing_keys.push(key.to_string());
            }
        }
    }

    Ok(Some(SessionRemoveResult {
        removed_ids,
        missing_keys,
    }))
}

pub fn create_session(provider: ProviderType, languages: Vec<Language>) -> Result<Session> {
    let supported_languages = to_supported_languages(provider, &languages)?;

    let session_id = storage::generate_id(provider, &supported_languages);

    let session = Session {
        id: session_id,
        provider,
        languages: supported_languages,
    };

    storage::save_session(&session)?;

    Ok(session)
}
