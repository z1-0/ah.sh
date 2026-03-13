use crate::error::{AppError, Result};
use crate::warning::AppWarning;
use clap::ValueEnum;
use serde_json::from_str;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use self::dev_templates::DevTemplatesProvider;
use self::devenv::DevenvProvider;

pub mod dev_templates;
pub mod devenv;

type LanguageAliases = HashMap<String, HashMap<String, String>>;
static LANGUAGE_ALIASES: OnceLock<Result<LanguageAliases>> = OnceLock::new();

fn parse_aliases(json: &str) -> Result<LanguageAliases> {
    from_str(json).map_err(|e| AppError::Generic(format!("Failed to parse language aliases: {e}")))
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProviderType {
    Devenv,
    DevTemplates,
}

impl ProviderType {
    pub fn into_shell_provider(self) -> Box<dyn ShellProvider> {
        match self {
            ProviderType::Devenv => Box::new(DevenvProvider),
            ProviderType::DevTemplates => Box::new(DevTemplatesProvider),
        }
    }
}

pub struct EnsureFilesResult {
    pub warnings: Vec<AppWarning>,
}

pub trait ShellProvider {
    fn name(&self) -> &str;
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult>;
    fn get_supported_languages(&self) -> Result<Vec<String>>;

    fn normalize_language(&self, lang: &str) -> String {
        // Backwards-compatible API: callers that want fail-fast should use
        // `try_normalize_lang_for_provider`.
        normalize_lang_for_provider(self.name(), lang)
    }
}

pub fn try_normalize_lang_for_provider(provider_name: &str, lang: &str) -> Result<String> {
    let aliases = LANGUAGE_ALIASES.get_or_init(|| {
        let aliases_json = include_str!("../assets/language_aliases.json");
        parse_aliases(aliases_json)
    });

    let aliases = match aliases {
        Ok(v) => v,
        Err(e) => return Err(AppError::Generic(e.to_string())),
    };

    Ok(aliases
        .get(lang)
        .and_then(|m| m.get(provider_name))
        .cloned()
        .unwrap_or_else(|| lang.to_owned()))
}

pub fn normalize_lang_for_provider(provider_name: &str, lang: &str) -> String {
    // Legacy behavior: if aliases parsing fails, fall back to identity.
    // Fail-fast call sites should use `try_normalize_lang_for_provider`.
    try_normalize_lang_for_provider(provider_name, lang).unwrap_or_else(|_| lang.to_owned())
}

pub fn validate_languages(languages: &[String], supported: &[String]) -> Result<()> {
    let supported_set: std::collections::HashSet<_> = supported.iter().collect();
    let invalids: Vec<_> = languages
        .iter()
        .filter(|language| !supported_set.contains(language))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(())
    } else {
        Err(AppError::UnsupportedLanguages(invalids))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_aliases_returns_err_for_invalid_json() {
        let err = super::parse_aliases("not json").expect_err("should error");
        assert!(
            err.to_string().contains("language aliases"),
            "unexpected error: {err}"
        );
    }
}
