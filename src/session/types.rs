use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::time::SystemTime;

pub const SESSION_ID_LEN: usize = 8;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid session key: {0}")]
    InvalidSelector(String),

    #[error("session '{0}' not found")]
    NotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub languages: Vec<String>,
    pub provider: String,
    pub created_at: u64,
}

impl Session {
    pub fn new(id: String, languages: Vec<String>, provider: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            languages,
            provider,
            created_at,
        }
    }
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
    type Err = AppError;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(SessionError::InvalidSelector(
                "session target cannot be empty".to_string(),
            )
            .into());
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input
                .parse::<usize>()
                .map_err(|_| SessionError::InvalidSelector("invalid session index".to_string()))?;
            if index == 0 {
                return Err(SessionError::InvalidSelector(
                    "session index must be greater than 0".to_string(),
                )
                .into());
            }
            return Ok(SessionKey::Index(index));
        }

        if !input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SessionError::InvalidSelector(
                "session id must contain only hexadecimal characters".to_string(),
            )
            .into());
        }

        if input.len() != SESSION_ID_LEN {
            return Err(SessionError::InvalidSelector(format!(
                "session id must be exactly {} hexadecimal characters",
                SESSION_ID_LEN
            ))
            .into());
        }

        Ok(SessionKey::Id(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_key_from_str_empty_is_err() {
        let err = SessionKey::from_str("").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cannot be empty"), "{msg}");
    }

    #[test]
    fn session_key_from_str_zero_index_is_err() {
        let err = SessionKey::from_str("0").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("greater than 0"), "{msg}");
    }

    #[test]
    fn session_key_from_str_one_is_ok_index() {
        let key = SessionKey::from_str("1").unwrap();
        assert_eq!(key, SessionKey::Index(1));
    }

    #[test]
    fn session_key_from_str_very_large_number_is_err() {
        let err = SessionKey::from_str("999999999999999999999999").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid session index"), "{msg}");
    }

    #[test]
    fn session_key_from_str_non_hex_is_err() {
        let err = SessionKey::from_str("zzzzzzzz").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("hexadecimal"), "{msg}");
    }

    #[test]
    fn session_key_from_str_hex_wrong_len_is_err() {
        for input in ["deadbeef00", "dead"] {
            let err = SessionKey::from_str(input).unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("exactly"), "{input}: {msg}");
        }
    }

    #[test]
    fn session_key_from_str_valid_hex_is_ok_id() {
        let key = SessionKey::from_str("deadbeef").unwrap();
        assert_eq!(key, SessionKey::Id("deadbeef".to_string()));
    }
}
