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
    fn normalize_language(&self, lang: &str) -> String {
        normalize_lang_for_provider(self.name(), lang)
    }
}

pub fn normalize_lang_for_provider(provider_name: &str, lang: &str) -> String {
    let aliases_json = include_str!("../assets/language_aliases.json");
    let aliases: HashMap<String, HashMap<String, String>> =
        from_str(aliases_json).unwrap_or_default();

    aliases
        .get(lang)
        .and_then(|m| m.get(provider_name))
        .cloned()
        .unwrap_or_else(|| lang.to_owned())
}
