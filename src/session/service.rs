use crate::error::{AppError, Result};
use crate::provider::{EnsureFilesResult, ProviderType, provider_info, validate_languages};
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

fn normalize_and_dedup_languages(
    provider_type: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    let provider = provider_info(provider_type);
    let mapped_langs = languages
        .iter()
        .map(|language| provider.normalize_language(language))
        .collect::<Result<Vec<_>>>()?;

    let mut deduped_langs = mapped_langs;
    let mut seen = HashSet::new();
    deduped_langs.retain(|language| seen.insert(language.clone()));

    Ok(deduped_langs)
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
        let provider_metadata = provider_info(provider_type);
        let provider_name = provider_metadata.name();
        let deduped_langs = normalize_and_dedup_languages(provider_type, &languages)?;

        if deduped_langs.is_empty() {
            return Err(AppError::Generic(
                "No languages specified. Use 'ah use <langs>' or 'ah session list'".to_string(),
            ));
        }

        let mut warnings: Vec<AppWarning> = Vec::new();

        let supported_langs = provider_metadata.supported_languages()?;
        validate_languages(&deduped_langs, &supported_langs)?;

        let session_id = storage::generate_id(provider_name, &deduped_langs);
        let session_dir = storage::get_session_dir()?.join(&session_id);
        let flake_path = session_dir.join("flake.nix");
        if flake_path.exists() {
            return Ok(CreateSessionResult {
                session_dir,
                warnings,
            });
        }
        std::fs::create_dir_all(&session_dir)?;

        let provider = provider_type.into_shell_provider();
        let EnsureFilesResult {
            warnings: provider_warnings,
        } = provider.ensure_files(&deduped_langs, &session_dir)?;
        warnings.extend(provider_warnings);

        let session = Session::new(session_id, deduped_langs, provider_name.to_string());
        storage::save_session(&session)?;

        Ok(CreateSessionResult {
            session_dir,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_and_dedup_languages;
    use crate::error::AppError;
    use crate::provider::{ProviderType, provider_info, validate_languages};
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn normalize_and_dedup_preserves_first_seen_order() {
        let languages = vec![
            "js".to_string(),
            "javascript".to_string(),
            "py".to_string(),
            "python".to_string(),
            "js".to_string(),
        ];

        let normalized = normalize_and_dedup_languages(ProviderType::Devenv, &languages).unwrap();

        assert_eq!(
            normalized,
            vec!["javascript".to_string(), "python".to_string()]
        );
    }

    #[test]
    fn unsupported_normalized_languages_preserve_error_shape() {
        let normalized = normalize_and_dedup_languages(
            ProviderType::Devenv,
            &["totally-not-a-language".to_string()],
        )
        .unwrap();
        let supported = provider_info(ProviderType::Devenv)
            .supported_languages()
            .unwrap();

        let err = validate_languages(&normalized, &supported).unwrap_err();
        assert!(matches!(
            err,
            AppError::UnsupportedLanguages(ref invalids)
            if invalids == &vec!["totally-not-a-language".to_string()]
        ));
    }

    #[test]
    fn create_session_does_not_overwrite_existing_flake() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("ah-test-cache-{ts}"));
        fs::create_dir_all(&temp_root).unwrap();
        unsafe {
            env::set_var("XDG_CACHE_HOME", &temp_root);
        }

        let provider_type = ProviderType::Devenv;
        let provider_name = provider_info(provider_type).name();
        let languages = vec!["rust".to_string()];
        let session_id = crate::session::storage::generate_id(provider_name, &languages);
        let session_dir = temp_root
            .join("ah")
            .join("ah")
            .join("sessions")
            .join(&session_id);
        fs::create_dir_all(&session_dir).unwrap();

        let flake_path = session_dir.join("flake.nix");
        let marker = "# marker flake";
        fs::write(&flake_path, marker).unwrap();

        let result = super::SessionService::create_session(provider_type, languages).unwrap();
        assert_eq!(result.session_dir, session_dir);

        let content = fs::read_to_string(&flake_path).unwrap();
        assert_eq!(content, marker);
    }
}
