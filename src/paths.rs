use crate::error::Result;
use std::path::PathBuf;

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_xdg_data_dir() -> Result<PathBuf> {
    let base_dir = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .map_err(|_| {
            crate::error::AhError::Generic(
                "Could not find XDG_DATA_HOME or HOME environment variable".to_string(),
            )
        })?;

    Ok(base_dir.join(PROGRAM_NAME))
}
