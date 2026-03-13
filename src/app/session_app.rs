use crate::error::Result;
use crate::session::{Session, SessionService};

pub(crate) struct SessionApp;

impl SessionApp {
    pub(crate) fn list_sessions() -> Result<Vec<Session>> {
        SessionService::list_sessions()
    }
}
