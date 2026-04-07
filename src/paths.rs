use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

fn app_name() -> &'static str {
    option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME"))
}

fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", app_name())
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
    let dir = get_cache_dir()?.join("sessions");
    Ok(dir)
}

fn get_current_session_path() -> Result<PathBuf> {
    let path = get_cache_dir()?.join("current_session");
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
