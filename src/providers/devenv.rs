use crate::error::Result;
use crate::providers::{ProviderAssetManager, ShellProvider};
use std::path::PathBuf;

pub struct DevenvProvider {
    manager: ProviderAssetManager,
}

impl Default for DevenvProvider {
    fn default() -> Self {
        Self {
            manager: ProviderAssetManager::new(
                "devenv",
                include_str!("../assets/providers/devenv/flake.nix"),
                include_str!("../assets/providers/devenv/supported_langs.json"),
            ),
        }
    }
}

impl ShellProvider for DevenvProvider {
    fn name(&self) -> &str {
        "devenv"
    }

    fn ensure_files(&self) -> Result<PathBuf> {
        self.manager.ensure_files()
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        self.manager.get_supported_languages()
    }

    fn normalize_language(&self, lang: &str) -> String {
        self.manager.normalize_language(lang)
    }
}
