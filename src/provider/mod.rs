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

pub(crate) type ProviderInputMap = HashMap<String, String>;
type ProviderLanguageMap = HashMap<String, Vec<String>>;
type LanguageMapByProvider = HashMap<String, ProviderLanguageMap>;
type InputMapByProvider = HashMap<String, ProviderInputMap>;

struct LanguageMaps {
    by_provider: LanguageMapByProvider,
    input_map: InputMapByProvider,
}

static LANGUAGE_MAPS: OnceLock<Result<LanguageMaps>> = OnceLock::new();

const PROVIDER_LANGUAGE_MAPS: [(&str, &str, &str); 2] = [
    (
        "devenv",
        include_str!("../assets/providers/devenv/supported_langs.json"),
        include_str!("../assets/providers/devenv/mapping_langs.json"),
    ),
    (
        "dev-templates",
        include_str!("../assets/providers/dev-templates/supported_langs.json"),
        include_str!("../assets/providers/dev-templates/mapping_langs.json"),
    ),
];

impl LanguageMaps {
    fn load() -> Result<Self> {
        let mut by_provider = HashMap::new();
        let mut input_map = HashMap::new();

        for (provider, supported_json, map_json) in PROVIDER_LANGUAGE_MAPS {
            let parsed_map = Self::parse_language_map(map_json)?;
            let inputs = Self::language_map_to_input_map(&parsed_map);
            let mut all_inputs = inputs;

            // Also include the raw supported languages
            let supported: Vec<String> = from_str(supported_json)
                .map_err(|e| AppError::Generic(format!("Failed to parse supported_langs: {e}")))?;
            for lang in supported {
                all_inputs.entry(lang.clone()).or_insert(lang);
            }

            by_provider.insert(provider.to_string(), parsed_map);
            input_map.insert(provider.to_string(), all_inputs);
        }

        Ok(Self {
            by_provider,
            input_map,
        })
    }

    fn global() -> Result<&'static Self> {
        let maps = LANGUAGE_MAPS.get_or_init(Self::load);
        maps.as_ref().map_err(|e| AppError::Generic(e.to_string()))
    }

    fn normalize(&self, provider_name: &str, input: &str) -> Result<String> {
        let map = self.input_map(provider_name)?;
        Ok(Self::map_language_with_input_map(input, map))
    }

    fn input_map(&self, provider_name: &str) -> Result<&ProviderInputMap> {
        self.input_map
            .get(provider_name)
            .ok_or_else(|| AppError::Generic(format!("Unsupported provider: {provider_name}")))
    }

    fn raw_map(&self, provider_name: &str) -> Result<&ProviderLanguageMap> {
        self.by_provider
            .get(provider_name)
            .ok_or_else(|| AppError::Generic(format!("Unsupported provider: {provider_name}")))
    }

    fn display_map(&self, provider_name: &str) -> Result<HashMap<String, Vec<String>>> {
        let map = self.raw_map(provider_name)?;
        Ok(Self::display_map_with_map(map))
    }

    fn display_map_with_map(map: &ProviderLanguageMap) -> HashMap<String, Vec<String>> {
        let mut by_mapped = HashMap::new();

        for (mapped, inputs) in map {
            let mut display_inputs: Vec<String> = inputs
                .iter()
                .filter(|input| *input != mapped)
                .cloned()
                .collect();
            display_inputs.sort();
            display_inputs.dedup();
            by_mapped.insert(mapped.clone(), display_inputs);
        }

        by_mapped
    }

    fn parse_language_map(json: &str) -> Result<ProviderLanguageMap> {
        from_str(json).map_err(|e| AppError::Generic(format!("Failed to parse language map: {e}")))
    }

    fn language_map_to_input_map(map: &ProviderLanguageMap) -> ProviderInputMap {
        let mut inputs = ProviderInputMap::new();

        for (mapped, candidates) in map {
            inputs.insert(mapped.clone(), mapped.clone());
            for candidate in candidates {
                inputs.insert(candidate.clone(), mapped.clone());
            }
        }

        inputs
    }

    fn map_language_with_input_map(input: &str, map: &ProviderInputMap) -> String {
        map.get(input).cloned().unwrap_or_else(|| input.to_string())
    }

    fn is_input_in_map(&self, input: &str) -> bool {
        self.input_map.values().any(|map| map.contains_key(input))
    }
}

pub fn is_maybe_language(input: &str) -> bool {
    LanguageMaps::global()
        .map(|maps| maps.is_input_in_map(input))
        .unwrap_or(false)
}

pub fn language_map_for_provider(provider_name: &str) -> Result<ProviderLanguageMap> {
    LanguageMaps::global()?.raw_map(provider_name).cloned()
}

pub fn map_language_for_provider(provider_name: &str, input: &str) -> Result<String> {
    LanguageMaps::global()?.normalize(provider_name, input)
}

pub fn language_map_for_display(provider_name: &str) -> Result<HashMap<String, Vec<String>>> {
    LanguageMaps::global()?.display_map(provider_name)
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
