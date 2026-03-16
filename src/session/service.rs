use crate::error::{AppError, Result};
use crate::providers::{EnsureFilesResult, ProviderType, validate_languages};
use crate::session::storage;
use crate::session::{Session, SessionError, SessionKey};
use crate::warning::AppWarning;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct SessionService;

pub struct CreateSessionResult {
    pub session_dir: PathBuf,
    pub warnings: Vec<AppWarning>,
}

pub struct SessionRemoveResult {
    pub removed_ids: Vec<String>,
    pub missing_keys: Vec<String>,
}

impl SessionService {
    pub fn list_sessions() -> Result<Vec<Session>> {
        storage::list_sessions()
    }

    pub fn resolve_session_dir(key: &SessionKey) -> Result<PathBuf> {
        let session = storage::find_session(key)?;
        Ok(storage::get_session_dir()?.join(&session.id))
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

    pub fn create_session(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<CreateSessionResult> {
        let provider = provider_type.into_shell_provider();

        let mapped_langs = languages
            .iter()
            .map(|language| provider.map_language(language))
            .collect::<Result<Vec<_>>>()?;

        let mut deduped_langs = mapped_langs;

        let mut seen = HashSet::new();
        deduped_langs.retain(|language| seen.insert(language.clone()));

        if deduped_langs.is_empty() {
            return Err(AppError::Generic(
                "No languages specified. Use 'ah use <langs>' or 'ah session list'".to_string(),
            ));
        }

        let mut warnings: Vec<AppWarning> = Vec::new();

        let supported_langs = provider.get_supported_languages()?;
        validate_languages(&deduped_langs, &supported_langs)?;

        let session_id = storage::generate_id(provider.name(), &deduped_langs);
        let session_dir = storage::get_session_dir()?.join(&session_id);
        std::fs::create_dir_all(&session_dir)?;

        let EnsureFilesResult {
            warnings: provider_warnings,
        } = provider.ensure_files(&deduped_langs, &session_dir)?;
        warnings.extend(provider_warnings);

        let session = Session::new(session_id, deduped_langs, provider.name().to_string());
        storage::save_session(&session)?;

        Ok(CreateSessionResult {
            session_dir,
            warnings,
        })
    }
}
