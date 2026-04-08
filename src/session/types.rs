use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::paths::get_session_dir;
use crate::provider::ProviderType;

pub const SESSION_ID_LEN: usize = 8;
pub const HISTORY_LIMIT: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub provider: ProviderType,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub path: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl Session {
    pub fn get_dir(&self) -> Result<PathBuf> {
        Ok(get_session_dir()?.join(&self.id))
    }
}

pub struct SessionRemoveResult {
    pub removed_ids: Vec<String>,
    pub missing_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionKey {
    Index(usize),
    Id(String),
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionKey::Index(i) => write!(f, "{i}"),
            SessionKey::Id(id) => write!(f, "{id}"),
        }
    }
}

impl FromStr for SessionKey {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            anyhow::bail!("invalid session key: session target cannot be empty");
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("invalid session key: invalid session index"))?;
            if index == 0 {
                anyhow::bail!("invalid session key: session index must be greater than 0");
            }
            return Ok(SessionKey::Index(index));
        }

        if !input.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!(
                "invalid session key: session id must contain only hexadecimal characters"
            );
        }

        if input.len() != SESSION_ID_LEN {
            anyhow::bail!(
                "invalid session key: session id must be exactly {} hexadecimal characters",
                SESSION_ID_LEN
            );
        }

        Ok(SessionKey::Id(input.to_string()))
    }
}
