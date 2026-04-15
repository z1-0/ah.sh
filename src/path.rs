use anyhow::Context;
use std::path::PathBuf;
use std::sync::LazyLock;

static PROJECT_DIRS: LazyLock<directories::ProjectDirs> = LazyLock::new(|| {
    directories::ProjectDirs::from("", "", crate::APP_NAME)
        .expect("Could not determine project directories")
});

pub fn get_cwd() -> anyhow::Result<PathBuf> {
    std::env::current_dir().context("failed to get current directory")
}

pub mod config {
    use super::*;

    pub const CONFIG_FILE: &str = "config.toml";

    fn get_dir() -> PathBuf {
        PROJECT_DIRS.config_dir().to_path_buf()
    }

    pub fn get_config_file() -> PathBuf {
        get_dir().join(CONFIG_FILE)
    }
}

pub mod cache {
    use super::*;

    pub const CURRENT_FILE: &str = "current_session";

    pub mod sessions {
        use super::*;

        pub const FLAKE_FILE: &str = "flake.nix";
        pub const FLAKE_LOCK_FILE: &str = "flake.lock";
        pub const HISTORY_FILE: &str = "history.json";
        pub const METADATA_FILE: &str = "metadata.json";
        pub const NIX_PROFILE_FILE: &str = "nix-profile";
        pub const SESSIONS_DIR: &str = "sessions";

        pub fn get_dir() -> PathBuf {
            super::get_dir().join(SESSIONS_DIR)
        }
    }

    fn get_dir() -> PathBuf {
        PROJECT_DIRS.cache_dir().to_path_buf()
    }

    fn get_current_session() -> PathBuf {
        get_dir().join(CURRENT_FILE)
    }

    pub fn read_current_session() -> anyhow::Result<Option<String>> {
        let path = get_current_session();
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn save_current_session(session_id: &str) -> anyhow::Result<()> {
        let path = get_current_session();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, session_id)?;
        Ok(())
    }

    pub fn clear_current_session() -> anyhow::Result<()> {
        let _ = std::fs::remove_file(get_current_session());
        Ok(())
    }
}
