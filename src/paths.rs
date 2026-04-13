use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

use crate::APP_NAME;

pub mod session {
    pub const CURRENT_FILE: &str = "current_session";
    pub const FLAKE_FILE: &str = "flake.nix";
    pub const FLAKE_LOCK_FILE: &str = "flake.lock";
    pub const HISTORY_FILE: &str = "history.json";
    pub const METADATA_FILE: &str = "metadata.json";
    pub const NIX_PROFILE_FILE: &str = "nix-profile";
    pub const SESSIONS_DIR: &str = "sessions";
}

pub mod config {
    use super::*;
    use anyhow::Result;

    /// Config file name
    pub const CONFIG_FILE: &str = "config.toml";

    /// Get full config file path: ~/.config/ah/config.toml
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join(CONFIG_FILE))
    }

    /// Get config directory: ~/.config/ah/
    pub fn get_config_dir() -> Result<PathBuf> {
        let project_dirs = get_project_dirs()?;
        Ok(project_dirs.config_dir().to_path_buf())
    }
}

fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", APP_NAME)
        .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))
}

pub fn get_data_dir() -> Result<PathBuf> {
    let project_dirs = get_project_dirs()?;

    Ok(project_dirs.data_dir().to_path_buf())
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let project_dirs = get_project_dirs()?;

    Ok(project_dirs.cache_dir().to_path_buf())
}

pub fn get_session_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?.join(session::SESSIONS_DIR);
    Ok(dir)
}

pub fn get_cwd() -> Result<PathBuf> {
    std::env::current_dir().context("failed to get current directory")
}

fn get_current_session_path() -> Result<PathBuf> {
    let path = get_cache_dir()?.join(session::CURRENT_FILE);
    Ok(path)
}

pub fn read_current_session() -> Result<Option<String>> {
    let path = get_current_session_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}

pub fn save_current_session(session_id: &str) -> Result<()> {
    let path = get_current_session_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, session_id)?;
    Ok(())
}

pub fn clear_current_session() -> Result<()> {
    let path = get_current_session_path()?;
    let _ = std::fs::remove_file(path);
    Ok(())
}
