pub mod flake_generator;

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

    fn ensure_files(&self, languages: &[String]) -> Result<PathBuf> {
        let dir = crate::paths::get_xdg_dir(crate::paths::XdgDir::Data)?
            .join("providers")
            .join(self.name());
        std::fs::create_dir_all(&dir)?;

        let flake_content = self::flake_generator::generate_devenv_flake(languages);

        let flake_path = dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(dir)
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        self.manager.get_supported_languages()
    }

    fn normalize_language(&self, lang: &str) -> String {
        self.manager.normalize_language(lang)
    }
}
