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

type ProviderLanguageMap = HashMap<String, Vec<String>>;
type LanguageMapByProvider = HashMap<String, ProviderLanguageMap>;
pub(crate) type ProviderInputMap = HashMap<String, String>;
type InputMapByProvider = HashMap<String, ProviderInputMap>;

struct LanguageMaps {
    by_provider: LanguageMapByProvider,
    input_map: InputMapByProvider,
}

static LANGUAGE_MAPS: OnceLock<Result<LanguageMaps>> = OnceLock::new();

fn parse_language_map(json: &str) -> Result<ProviderLanguageMap> {
    from_str(json).map_err(|e| AppError::Generic(format!("Failed to parse language map: {e}")))
}

pub(crate) fn language_map_to_input_map(map: ProviderLanguageMap) -> ProviderInputMap {
    let mut inputs = ProviderInputMap::new();

    for (mapped, candidates) in map.iter() {
        inputs.insert(mapped.clone(), mapped.clone());
        for candidate in candidates {
            inputs.insert(candidate.clone(), mapped.clone());
        }
    }

    inputs
}

fn load_language_maps() -> Result<LanguageMaps> {
    let mut by_provider = HashMap::new();
    let mut input_map = HashMap::new();
    let providers = ["devenv", "dev-templates"];

    for provider in providers {
        let map_json = match provider {
            "devenv" => include_str!("../assets/providers/devenv/mapping_langs.json"),
            "dev-templates" => include_str!("../assets/providers/dev-templates/mapping_langs.json"),
            _ => return Err(AppError::Generic("Unknown provider".to_string())),
        };
        let parsed = parse_language_map(map_json)?;
        let inputs = language_map_to_input_map(parsed.clone());

        by_provider.insert(provider.to_string(), parsed);
        input_map.insert(provider.to_string(), inputs);
    }

    Ok(LanguageMaps {
        by_provider,
        input_map,
    })
}

pub fn language_map_for_provider(provider_name: &str) -> Result<ProviderLanguageMap> {
    let maps = LANGUAGE_MAPS.get_or_init(|| load_language_maps());
    let maps = maps
        .as_ref()
        .map_err(|e| AppError::Generic(e.to_string()))?;

    maps.by_provider
        .get(provider_name)
        .cloned()
        .ok_or_else(|| AppError::Generic(format!("Unsupported provider: {provider_name}")))
}

pub fn input_map_for_provider(provider_name: &str) -> Result<ProviderInputMap> {
    let maps = LANGUAGE_MAPS.get_or_init(|| load_language_maps());
    let maps = maps
        .as_ref()
        .map_err(|e| AppError::Generic(e.to_string()))?;

    maps.input_map
        .get(provider_name)
        .cloned()
        .ok_or_else(|| AppError::Generic(format!("Unsupported provider: {provider_name}")))
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

    fn map_language(&self, input: &str) -> Result<String> {
        map_language_for_provider(self.name(), input)
    }
}

fn map_language_with_input_map(input: &str, map: &Result<ProviderInputMap>) -> Result<String> {
    let map = map.as_ref().map_err(|e| AppError::Generic(e.to_string()))?;

    Ok(map.get(input).cloned().unwrap_or_else(|| input.to_string()))
}

pub fn map_language_for_provider(provider_name: &str, input: &str) -> Result<String> {
    let map = input_map_for_provider(provider_name)?;
    map_language_with_input_map(input, &Ok(map))
}

pub fn language_map_for_display(provider_name: &str) -> Result<HashMap<String, Vec<String>>> {
    let map = language_map_for_provider(provider_name)?;
    Ok(language_map_for_display_with_map(map))
}

pub(crate) fn language_map_for_display_with_map(
    map: ProviderLanguageMap,
) -> HashMap<String, Vec<String>> {
    let mut by_mapped = HashMap::new();

    for (mapped, inputs) in map {
        let mut display_inputs: Vec<String> = inputs
            .into_iter()
            .filter(|input| input != &mapped)
            .collect();
        display_inputs.sort();
        display_inputs.dedup();
        by_mapped.insert(mapped, display_inputs);
    }

    by_mapped
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
    fn parse_language_map_returns_err_for_invalid_json() {
        let err = super::parse_language_map("not json").expect_err("should error");
        assert!(
            err.to_string().contains("language map"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn map_language_returns_err_when_map_source_is_err() {
        let map: super::Result<super::ProviderInputMap> =
            Err(AppError::Generic("map source failed".to_string()));

        let err = super::map_language_with_input_map("rust", &map).expect_err("should error");
        assert!(err.to_string().contains("map source failed"));
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
    fn map_language_with_map_returns_mapped_value_when_present() {
        let map_json = r#"{
  "javascript": ["js", "javascript"]
}"#;
        let map = super::parse_language_map(map_json).expect("should parse");
        let input_map = super::language_map_to_input_map(map);

        let mapped = super::map_language_with_input_map("js", &Ok(input_map)).expect("should map");

        assert_eq!(mapped, "javascript");
    }

    #[test]
    fn map_language_with_map_returns_original_when_no_mapping_found() {
        let map_json = r#"{
  "javascript": ["js", "javascript"]
}"#;
        let map = super::parse_language_map(map_json).expect("should parse");
        let input_map = super::language_map_to_input_map(map);

        let mapped = super::map_language_with_input_map("go", &Ok(input_map)).expect("should map");

        assert_eq!(mapped, "go");
    }

    #[test]
    fn map_language_for_provider_returns_original_for_unknown_language() {
        let lang = "__no_such_lang__";
        let mapped = super::map_language_for_provider("devenv", lang).expect("should ok");
        assert_eq!(mapped, lang);
    }

    #[test]
    fn language_map_for_display_filters_self_mappings() {
        let map_json = r#"{
  "javascript": ["js", "javascript"]
}"#;
        let map = super::parse_language_map(map_json).expect("should parse");
        let display = super::language_map_for_display_with_map(map);

        assert_eq!(display.get("javascript"), Some(&vec!["js".to_string()]));
    }
}
