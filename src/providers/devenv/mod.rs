pub mod flake_generator;

use crate::error::Result;
use crate::providers::ShellProvider;
use std::path::Path;

pub struct DevenvProvider;

impl Default for DevenvProvider {
    fn default() -> Self {
        Self
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
        let langs_json = include_str!("../../assets/providers/devenv/supported_langs.json");
        let langs = serde_json::from_str(langs_json)?;
        Ok(langs)
    }
}
