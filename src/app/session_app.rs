use crate::error::Result;
use crate::session::{Session, SessionService};

pub struct SessionApp;

impl SessionApp {
    pub fn list_sessions() -> Result<Vec<Session>> {
        SessionService::list_sessions()
    }
}
