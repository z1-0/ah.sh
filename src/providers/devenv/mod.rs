pub mod flake_generator;

use crate::error::Result;
use crate::providers::{ProviderAssetManager, ShellProvider};
use std::path::Path;

pub struct DevenvProvider {
    manager: ProviderAssetManager,
}

impl Default for DevenvProvider {
    fn default() -> Self {
        Self {
            manager: ProviderAssetManager::new(
                "devenv",
                "", // Now dynamically generated in ensure_files
                include_str!("../../assets/providers/devenv/supported_langs.json"),
            ),
        }
    }
}

impl ShellProvider for DevenvProvider {
    fn name(&self) -> &str {
        "devenv"
    }

    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<()> {
        let flake_content = self::flake_generator::generate_devenv_flake(languages);

        let flake_path = target_dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(())
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        self.manager.get_supported_languages()
    }

    fn normalize_language(&self, lang: &str) -> String {
        self.manager.normalize_language(lang)
    }
}
