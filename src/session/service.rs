use crate::paths::get_session_dir;
use crate::provider::{ProviderType, normalize_language, supported_languages, validate_languages};
use crate::session::storage;
use crate::session::types::{Session, SessionKey, SessionRemoveResult};
use anyhow::Result;
use std::collections::HashSet;

pub struct SessionService;

fn normalize_and_dedup_languages(
    provider: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    let mut mapped_langs = languages
        .iter()
        .map(|language| normalize_language(provider, language))
        .collect::<Result<Vec<_>>>()?;

    mapped_langs.sort_unstable();
    mapped_langs.dedup();

    Ok(mapped_langs)
}

fn get_provider_supported_langs(provider: ProviderType) -> Result<Vec<String>> {
    supported_languages(provider)
}

impl SessionService {
    /// Find an existing session by provider + language list
    pub fn find_session(provider: ProviderType, languages: &[String]) -> Result<Option<Session>> {
        let deduped_langs = normalize_and_dedup_languages(provider, languages)?;

        if deduped_langs.is_empty() {
            return Ok(None);
        }

        let session_id = storage::generate_id(provider, &deduped_langs);
        let meta_path = get_session_dir()?.join(&session_id).join("metadata.json");

        let content = match std::fs::read_to_string(&meta_path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let session = match serde_json::from_str::<Session>(&content) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        Ok(Some(session))
    }

    pub fn list_sessions() -> Result<Vec<Session>> {
        storage::list_sessions()
    }

    pub fn resolve_session_dir(key: &SessionKey) -> Result<Session> {
        storage::find_session(key)
    }

    pub fn clear_sessions() -> Result<usize> {
        storage::clear_sessions()
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
            match storage::resolve_session(&sessions, key) {
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
                    tracing::warn!("failed to resolve session: {}", e);
                    missing_keys.push(key.to_string());
                }
            }
        }

        Ok(Some(SessionRemoveResult {
            removed_ids,
            missing_keys,
        }))
    }

    /// Create a new session (assumes session doesn't exist - call find_session first)
    pub fn create_session(provider: ProviderType, languages: Vec<String>) -> Result<Session> {
        let deduped_langs = normalize_and_dedup_languages(provider, &languages)?;

        if deduped_langs.is_empty() {
            anyhow::bail!("No languages specified. Use 'ah use <langs>' or 'ah session list'");
        }

        let supported_langs = get_provider_supported_langs(provider)?;
        validate_languages(&deduped_langs, &supported_langs)?;

        let session_id = storage::generate_id(provider, &deduped_langs);

        let session = Session {
            id: session_id,
            provider,
            languages: deduped_langs,
        };

        storage::save_session(&session)?;

        Ok(session)
    }
}
