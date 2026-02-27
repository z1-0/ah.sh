pub mod dev_templates;
pub mod devenv;

use crate::error::Result;
use serde_json::from_str;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait ShellProvider {
    fn name(&self) -> &str;
    fn ensure_files(&self) -> Result<PathBuf>;
    fn get_supported_languages(&self) -> Result<Vec<String>>;
    fn normalize_language(&self, lang: &str) -> String;
}

/// Helper for common provider operations to reduce code duplication
pub struct ProviderAssetManager {
    name: String,
    flake_content: &'static str,
    langs_json: &'static str,
}

impl ProviderAssetManager {
    pub fn new(name: &str, flake: &'static str, langs: &'static str) -> Self {
        Self {
            name: name.to_string(),
            flake_content: flake,
            langs_json: langs,
        }
    }

    pub fn ensure_files(&self) -> Result<PathBuf> {
        let dir = crate::paths::get_xdg_data_dir()?
            .join("providers")
            .join(&self.name);
        std::fs::create_dir_all(&dir)?;

        let flake_path = dir.join("flake.nix");
        std::fs::write(flake_path, self.flake_content)?;

        Ok(dir)
    }

    pub fn get_supported_languages(&self) -> Result<Vec<String>> {
        let langs = from_str(self.langs_json)?;
        Ok(langs)
    }

    pub fn normalize_language(&self, lang: &str) -> String {
        let aliases_json = include_str!("../assets/language_aliases.json");
        let aliases: HashMap<String, HashMap<String, String>> =
            from_str(aliases_json).unwrap_or_default();

        aliases
            .get(lang)
            .and_then(|m| m.get(&self.name))
            .cloned()
            .unwrap_or_else(|| lang.to_owned())
    }
}
