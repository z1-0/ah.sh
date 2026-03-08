pub mod flake_generator;

use crate::error::{AppError, Result};
use crate::providers::ShellProvider;
use std::path::Path;
use std::sync::OnceLock;

pub struct DevenvProvider;

static SUPPORTED_LANGUAGES: OnceLock<std::result::Result<Vec<String>, String>> = OnceLock::new();

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
        let langs = SUPPORTED_LANGUAGES.get_or_init(|| {
            let langs_json = include_str!("../../assets/providers/devenv/supported_langs.json");
            serde_json::from_str(langs_json).map_err(|e| e.to_string())
        });

        langs.clone().map_err(AppError::Provider)
    }
}
