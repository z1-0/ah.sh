use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

pub const SESSION_ID_LEN: usize = 8;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub session_dir: PathBuf,
    pub provider: String,
    pub languages: Vec<String>,
}

pub struct CreateSessionResult {
    pub session: Session,
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
            return Err(anyhow::anyhow!(
                "invalid session key: session target cannot be empty"
            ));
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("invalid session key: invalid session index"))?;
            if index == 0 {
                return Err(anyhow::anyhow!(
                    "invalid session key: session index must be greater than 0"
                ));
            }
            return Ok(SessionKey::Index(index));
        }

        if !input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow::anyhow!(
                "invalid session key: session id must contain only hexadecimal characters"
            ));
        }

        if input.len() != SESSION_ID_LEN {
            return Err(anyhow::anyhow!(
                "invalid session key: session id must be exactly {} hexadecimal characters",
                SESSION_ID_LEN
            ));
        }

        Ok(SessionKey::Id(input.to_string()))
    }
}
