use crate::error::Result;
use crate::providers::{ProviderAssetManager, ShellProvider};
use std::path::PathBuf;

pub struct DevTemplatesProvider {
    manager: ProviderAssetManager,
}

impl Default for DevTemplatesProvider {
    fn default() -> Self {
        Self {
            manager: ProviderAssetManager::new(
                "dev-templates",
                include_str!("../assets/providers/dev-templates/flake.nix"),
                include_str!("../assets/providers/dev-templates/supported_langs.json"),
            ),
        }
    }
}

impl ShellProvider for DevTemplatesProvider {
    fn name(&self) -> &str {
        "dev-templates"
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
