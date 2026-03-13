use crate::error::Result;
use crate::session::{Session, SessionKey, SessionRemoveResult, SessionService};

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
}
