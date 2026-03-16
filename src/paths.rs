use crate::error::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_data_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", PROGRAM_NAME).ok_or_else(|| {
        crate::error::AppError::Generic("Could not determine project directories".to_string())
    })?;

    Ok(project_dirs.data_dir().join(PROGRAM_NAME))
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", PROGRAM_NAME).ok_or_else(|| {
        crate::error::AppError::Generic("Could not determine project directories".to_string())
    })?;

    Ok(project_dirs.cache_dir().join(PROGRAM_NAME))
}
