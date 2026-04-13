fn get_project_dirs() -> anyhow::Result<directories::ProjectDirs> {
    directories::ProjectDirs::from("", "", crate::APP_NAME)
        .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))
}

pub fn get_cwd() -> anyhow::Result<std::path::PathBuf> {
    anyhow::Context::context(std::env::current_dir(), "failed to get current directory")
}

pub mod config {
    use anyhow::Result;
    use std::path::PathBuf;

    use crate::paths::get_project_dirs;

    pub const CONFIG_FILE: &str = "config.toml";

    fn get_config_dir() -> Result<PathBuf> {
        let project_dirs = get_project_dirs()?;
        Ok(project_dirs.config_dir().to_path_buf())
    }

    pub fn get_config_file() -> Result<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join(CONFIG_FILE))
    }
}

pub mod cache {
    use anyhow::Result;
    use std::path::PathBuf;

    use crate::paths::get_project_dirs;

    pub const CURRENT_FILE: &str = "current_session";

    pub mod session {
        use anyhow::Result;
        use std::path::PathBuf;

        use crate::paths::cache::get_cache_dir;

        pub const FLAKE_FILE: &str = "flake.nix";
        pub const FLAKE_LOCK_FILE: &str = "flake.lock";
        pub const HISTORY_FILE: &str = "history.json";
        pub const METADATA_FILE: &str = "metadata.json";
        pub const NIX_PROFILE_FILE: &str = "nix-profile";
        pub const SESSIONS_DIR: &str = "sessions";

        pub fn get_session_dir() -> Result<PathBuf> {
            let dir = get_cache_dir()?.join(SESSIONS_DIR);
            Ok(dir)
        }
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let project_dirs = get_project_dirs()?;
        Ok(project_dirs.cache_dir().to_path_buf())
    }

    fn get_current_session_path() -> Result<PathBuf> {
        let path = get_cache_dir()?.join(CURRENT_FILE);
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
}
