pub mod dev_templates;
pub mod devenv;

use crate::error::Result;
use serde_json::from_str;
use std::collections::HashMap;
use std::path::Path;

pub trait ShellProvider {
    fn name(&self) -> &str;
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<()>;
    fn get_supported_languages(&self) -> Result<Vec<String>>;
    fn normalize_language(&self, lang: &str) -> String;
}

/// Helper for common provider operations to reduce code duplication
pub struct ProviderAssetManager {
    name: String,
    langs_json: &'static str,
}

impl ProviderAssetManager {
    pub fn new(name: &str, _flake: &'static str, langs: &'static str) -> Self {
        Self {
            name: name.to_string(),
            langs_json: langs,
        }
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
