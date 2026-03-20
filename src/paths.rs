use crate::error::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", PROGRAM_NAME).ok_or_else(|| {
        crate::error::AppError::Generic("Could not determine project directories".to_string())
    })
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
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn get_attrs_cache_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?.join("dev-templates").join("attrs");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}
