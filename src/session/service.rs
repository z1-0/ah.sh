use crate::paths::get_session_dir;
use crate::provider::{
    ProviderType, normalize_language, provider_name, supported_languages, validate_languages,
};
use crate::session::storage;
use crate::session::types::{CreateSessionResult, Session, SessionKey, SessionRemoveResult};
use anyhow::Result;
use std::collections::HashSet;

pub struct SessionService;

fn normalize_and_dedup_languages(
    provider_type: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    let mapped_langs = languages
        .iter()
        .map(|language| normalize_language(provider_type, language))
        .collect::<Result<Vec<_>>>()?;

    let mut deduped_langs = mapped_langs;
    let mut seen = HashSet::new();
    deduped_langs.retain(|language: &String| seen.insert(language.clone()));

    Ok(deduped_langs)
}

impl SessionService {
    /// Find an existing session by provider + language list
    pub fn find_session(
        provider_type: ProviderType,
        languages: &[String],
    ) -> Result<Option<Session>> {
        let provider_name = provider_name(provider_type);
        let deduped_langs = normalize_and_dedup_languages(provider_type, languages)?;

        if deduped_langs.is_empty() {
            return Ok(None);
        }

        let session_id = storage::generate_id(provider_name, &deduped_langs);
        let session_dir = get_session_dir()?.join(&session_id);
        let flake_path = session_dir.join("flake.nix");

        if !flake_path.exists() {
            return Ok(None);
        }

        // Session exists, read metadata
        let sessions = storage::list_sessions()?;
        let session = sessions
            .into_iter()
            .find(|s| s.id == session_id)
            .unwrap_or(Session {
                id: session_id,
                session_dir: session_dir.clone(),
                provider: provider_name.to_string(),
                languages: deduped_langs,
            });

        Ok(Some(session))
    }

    pub fn list_sessions() -> Result<Vec<Session>> {
        storage::list_sessions()
    }

    pub fn resolve_session_dir(key: &SessionKey) -> Result<std::path::PathBuf> {
        let session = storage::find_session(key)?;
        Ok(session.session_dir)
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
                        let session_id = session.id;
                        if storage::delete_session(&session_id)? {
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
    pub fn create_session(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<CreateSessionResult> {
        let provider_name = provider_name(provider_type);
        let deduped_langs = normalize_and_dedup_languages(provider_type, &languages)?;

        if deduped_langs.is_empty() {
            anyhow::bail!("No languages specified. Use 'ah use <langs>' or 'ah session list'");
        }

        let supported_langs = supported_languages(provider_type)?;
        validate_languages(&deduped_langs, &supported_langs)?;

        let session_id = storage::generate_id(provider_name, &deduped_langs);
        let session_dir = get_session_dir()?.join(&session_id);

        // Note: caller should check if session exists first via find_session
        // This function assumes a new session needs to be created
        std::fs::create_dir_all(&session_dir)?;

        let provider = provider_type.into_shell_provider();
        provider.ensure_files(&deduped_langs, &session_dir)?;

        // Save persisted session metadata
        let session = Session {
            id: session_id,
            session_dir,
            provider: provider_name.to_string(),
            languages: deduped_langs,
        };
        storage::save_session(&session)?;

        Ok(CreateSessionResult { session })
    }
}
