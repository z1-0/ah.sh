use crate::error::{AppError, Result};
use crate::providers::{validate_languages, ProviderType};
use crate::session::{
    clear_sessions, delete_session, find_session, generate_id, get_session_dir, list_sessions,
    resolve_session, save_session, Session, SessionError, SessionKey,
};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct SessionService;

pub struct SessionRemoveResult {
    pub removed_ids: Vec<String>,
    pub missing_keys: Vec<String>,
}

impl SessionService {
    pub fn list_sessions() -> Result<Vec<Session>> {
        list_sessions()
    }

    pub fn resolve_session_dir(key: &SessionKey) -> Result<PathBuf> {
        let session = find_session(key)?;
        Ok(get_session_dir()?.join(&session.id))
    }

    pub fn clear_sessions() -> Result<usize> {
        clear_sessions()
    }

    pub fn remove_sessions(keys: &[SessionKey]) -> Result<Option<SessionRemoveResult>> {
        if keys.is_empty() {
            return Ok(None);
        }

        let sessions = list_sessions()?;
        if sessions.is_empty() {
            return Ok(None);
        }

        let mut removed_ids = Vec::new();
        let mut missing_keys = Vec::new();
        let mut deduped_session_ids = HashSet::new();

        for key in keys {
            match resolve_session(&sessions, key) {
                Ok(session) => {
                    if deduped_session_ids.insert(session.id.clone()) {
                        let session_id = session.id;
                        if delete_session(&session_id)? {
                            removed_ids.push(session_id);
                        } else {
                            missing_keys.push(session_id);
                        }
                    }
                }
                Err(AppError::Session(SessionError::NotFound(missing_input))) => {
                    missing_keys.push(missing_input)
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Some(SessionRemoveResult {
            removed_ids,
            missing_keys,
        }))
    }

    pub fn create_session(provider_type: ProviderType, languages: Vec<String>) -> Result<PathBuf> {
        let provider = provider_type.into_shell_provider();

        let mut normalized_langs = languages
            .iter()
            .map(|language| provider.normalize_language(language))
            .collect::<Vec<_>>();

        let mut seen = HashSet::new();
        normalized_langs.retain(|language| seen.insert(language.clone()));

        if normalized_langs.is_empty() {
            return Err(AppError::Generic(
                "No languages specified. Use 'ah <langs>' or 'ah session list'".to_string(),
            ));
        }

        let supported_langs = provider.get_supported_languages()?;
        validate_languages(&normalized_langs, &supported_langs)?;

        let session_id = generate_id(provider.name(), &normalized_langs);
        let session_dir = get_session_dir()?.join(&session_id);
        std::fs::create_dir_all(&session_dir)?;

        provider.ensure_files(&normalized_langs, &session_dir)?;

        let session = Session::new(session_id, normalized_langs, provider.name().to_string());
        save_session(&session)?;

        Ok(session_dir)
    }
}
