use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::path;
use crate::provider::ProviderType;

pub const SESSION_ID_LEN: usize = 8;
pub const HISTORY_LIMIT: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub provider: ProviderType,
    pub languages: Vec<String>,
}

impl Session {
    pub fn get_dir(&self) -> PathBuf {
        path::cache::sessions::get_dir().join(&self.id)
    }
}

pub struct SessionRemoveResult {
    pub removed_ids: Vec<String>,
    pub missing_keys: Vec<String>,
}

#[derive(strum::Display, Debug, Clone, PartialEq, Eq)]
pub enum SessionKey {
    Index(usize),
    Id(String),
}

impl FromStr for SessionKey {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        if input.is_empty() {
            anyhow::bail!("session target cannot be empty");
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input.parse::<usize>()?;
            if index == 0 {
                anyhow::bail!("session index must be greater than 0");
            }
            return Ok(SessionKey::Index(index));
        }

        if input.len() != 8 || !input.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("session id must be exactly 8 hexadecimal characters");
        }

        Ok(SessionKey::Id(input.to_lowercase()))
    }
}
