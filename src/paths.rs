use crate::error::Result;
use std::path::PathBuf;

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub enum XdgDir {
    Data,
    Cache,
}

pub fn get_xdg_dir(dir_type: XdgDir) -> Result<PathBuf> {
    let (env_var, default_suffix) = match dir_type {
        XdgDir::Data => ("XDG_DATA_HOME", ".local/share"),
        XdgDir::Cache => ("XDG_CACHE_HOME", ".cache"),
    };

    let base_dir = std::env::var(env_var)
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(default_suffix)))
        .map_err(|_| {
            crate::error::AhError::Generic(format!(
                "Could not find {} or HOME environment variable",
                env_var
            ))
        })?;

    Ok(base_dir.join(PROGRAM_NAME))
}
