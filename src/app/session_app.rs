use crate::error::Result;
use crate::providers::ProviderType;
use crate::session::{
    CreateSessionResult, Session, SessionKey, SessionRemoveResult, SessionService,
};
use std::path::PathBuf;

pub(crate) struct SessionApp;

impl SessionApp {
    pub(crate) fn list_sessions() -> Result<Vec<Session>> {
        SessionService::list_sessions()
    }

    pub(crate) fn clear_sessions() -> Result<usize> {
        SessionService::clear_sessions()
    }

    pub(crate) fn remove_sessions(keys: &[SessionKey]) -> Result<Option<SessionRemoveResult>> {
        SessionService::remove_sessions(keys)
    }

    pub(crate) fn prepare_restore_session(key: &SessionKey) -> Result<PathBuf> {
        SessionService::resolve_session_dir(key)
    }

    pub(crate) fn prepare_create_session(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<CreateSessionResult> {
        SessionService::create_session(provider_type, languages)
    }
}
