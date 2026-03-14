use crate::error::{AppError, Result};
use crate::warning::AppWarning;
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

#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderType {
    Devenv,
    DevTemplates,
}

/// Target of `ah provider show`: select a provider, or choose all.
#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderKeyOrAll {
    Devenv,
    DevTemplates,
    All,
}

impl ProviderKeyOrAll {
    pub fn as_provider_type(&self) -> Option<ProviderType> {
        match self {
            ProviderKeyOrAll::Devenv => Some(ProviderType::Devenv),
            ProviderKeyOrAll::DevTemplates => Some(ProviderType::DevTemplates),
            ProviderKeyOrAll::All => None,
        }
    }
}

impl From<ProviderType> for ProviderKeyOrAll {
    fn from(value: ProviderType) -> Self {
        match value {
            ProviderType::Devenv => ProviderKeyOrAll::Devenv,
            ProviderType::DevTemplates => ProviderKeyOrAll::DevTemplates,
        }
    }
}

impl From<ProviderKeyOrAll> for Option<ProviderType> {
    fn from(value: ProviderKeyOrAll) -> Self {
        value.as_provider_type()
    }
}

pub fn language_aliases_by_canonical_for_provider(
    provider_name: &str,
) -> Result<HashMap<String, Vec<String>>> {
    let aliases = LANGUAGE_ALIASES.get_or_init(|| {
        let aliases_json = include_str!("../assets/language_aliases.json");
        parse_aliases(aliases_json)
    });

    let aliases = aliases
        .as_ref()
        .map_err(|e| AppError::Generic(e.to_string()))?;

    let mut by_canonical: HashMap<String, Vec<String>> = HashMap::new();

    for (alias, mapping) in aliases.iter() {
        let Some(canonical) = mapping.get(provider_name) else {
            continue;
        };

        // Avoid duplicating the canonical name in the alias list.
        if alias == canonical {
            continue;
        }

        by_canonical
            .entry(canonical.clone())
            .or_default()
            .push(alias.clone());
    }

    for aliases in by_canonical.values_mut() {
        aliases.sort();
        aliases.dedup();
    }

    Ok(by_canonical)
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

    fn normalize_language(&self, lang: &str) -> Result<String> {
        normalize_lang_for_provider(self.name(), lang)
    }
}

fn normalize_lang_for_provider_with_aliases(
    provider_name: &str,
    lang: &str,
    aliases: &Result<LanguageAliases>,
) -> Result<String> {
    let aliases = aliases
        .as_ref()
        .map_err(|e| AppError::Generic(e.to_string()))?;

    Ok(aliases
        .get(lang)
        .and_then(|m| m.get(provider_name))
        .cloned()
        .unwrap_or_else(|| lang.to_owned()))
}

pub fn normalize_lang_for_provider(provider_name: &str, lang: &str) -> Result<String> {
    let aliases = LANGUAGE_ALIASES.get_or_init(|| {
        let aliases_json = include_str!("../assets/language_aliases.json");
        parse_aliases(aliases_json)
    });

    normalize_lang_for_provider_with_aliases(provider_name, lang, aliases)
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
    use crate::error::AppError;

    #[test]
    fn parse_aliases_returns_err_for_invalid_json() {
        let err = super::parse_aliases("not json").expect_err("should error");
        assert!(
            err.to_string().contains("language aliases"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn normalize_returns_err_when_aliases_source_is_err() {
        let aliases: super::Result<super::LanguageAliases> =
            Err(AppError::Generic("aliases source failed".to_string()));

        let err = super::normalize_lang_for_provider_with_aliases("devenv", "rust", &aliases)
            .expect_err("should error");
        assert!(err.to_string().contains("aliases source failed"));
    }

    #[test]
    fn validate_languages_returns_ok_when_all_languages_supported() {
        let languages = vec!["rust".to_string(), "go".to_string()];
        let supported = vec!["rust".to_string(), "go".to_string()];

        super::validate_languages(&languages, &supported).expect("should be ok");
    }

    #[test]
    fn validate_languages_returns_err_for_unsupported_language() {
        let languages = vec!["rust".to_string(), "python".to_string()];
        let supported = vec!["rust".to_string(), "go".to_string()];

        let err = super::validate_languages(&languages, &supported).expect_err("should error");
        match err {
            AppError::UnsupportedLanguages(invalids) => {
                assert!(
                    invalids.contains(&"python".to_string()),
                    "invalids: {invalids:?}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn normalize_lang_for_provider_with_aliases_returns_mapped_value_when_present() {
        let aliases_json = r#"{
  "js": { "devenv": "javascript" }
}"#;
        let aliases = super::parse_aliases(aliases_json).expect("should parse");

        let normalized =
            super::normalize_lang_for_provider_with_aliases("devenv", "js", &Ok(aliases))
                .expect("should normalize");

        assert_eq!(normalized, "javascript");
    }

    #[test]
    fn normalize_lang_for_provider_with_aliases_returns_original_when_no_mapping_found() {
        let aliases_json = r#"{
  "js": { "devenv": "javascript" }
}"#;
        let aliases = super::parse_aliases(aliases_json).expect("should parse");

        // Different provider => no mapping => return original.
        let normalized =
            super::normalize_lang_for_provider_with_aliases("dev_templates", "js", &Ok(aliases))
                .expect("should normalize");

        assert_eq!(normalized, "js");
    }

    #[test]
    fn normalize_lang_for_provider_returns_original_for_unknown_language() {
        let lang = "__no_such_lang__";
        let normalized = super::normalize_lang_for_provider("devenv", lang).expect("should ok");
        assert_eq!(normalized, lang);
    }
}
