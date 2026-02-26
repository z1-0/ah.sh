use crate::error::Result;
use std::path::PathBuf;

pub fn get_ah_data_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").or_else(|_| {
        std::env::var("USERPROFILE")
    }).map_err(|_| crate::error::AhError::Generic("Could not find home directory".to_string()))?;

    Ok(PathBuf::from(home).join(".local/share/ah"))
}
