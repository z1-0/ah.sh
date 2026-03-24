use crate::provider::ProviderType;
use anyhow::{Context, Result};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) type ProviderInputMap = HashMap<String, String>;
pub(crate) type ProviderLanguageMap = HashMap<String, Vec<String>>;
type LanguageMapByProvider = HashMap<String, ProviderLanguageMap>;
type InputMapByProvider = HashMap<String, ProviderInputMap>;
type SupportedLanguagesByProvider = HashMap<String, Vec<String>>;

struct LanguageMaps {
    by_provider: LanguageMapByProvider,
    input_map: InputMapByProvider,
    supported_languages: SupportedLanguagesByProvider,
}

static LANGUAGE_MAPS: OnceLock<Result<LanguageMaps>> = OnceLock::new();

const PROVIDER_LANGUAGE_MAPS: [(ProviderType, &str, &str); 2] = [
    (
        ProviderType::Devenv,
        include_str!("../assets/providers/devenv/supported_langs.json"),
        include_str!("../assets/providers/devenv/mapping_langs.json"),
    ),
    (
        ProviderType::DevTemplates,
        include_str!("../assets/providers/dev-templates/supported_langs.json"),
        include_str!("../assets/providers/dev-templates/mapping_langs.json"),
    ),
];

impl LanguageMaps {
    fn load() -> Result<Self> {
        let mut by_provider = HashMap::new();
        let mut input_map = HashMap::new();
        let mut supported_languages = HashMap::new();

        for (provider_type, supported_json, map_json) in PROVIDER_LANGUAGE_MAPS {
            let provider = provider_type.to_string();
            let parsed_map = Self::parse_language_map(map_json)?;
            let inputs = Self::language_map_to_input_map(&parsed_map);
            let mut all_inputs = inputs;

            let supported: Vec<String> =
                from_str(supported_json).context("Failed to parse supported_langs")?;
            for lang in &supported {
                all_inputs
                    .entry(lang.clone())
                    .or_insert_with(|| lang.clone());
            }

            by_provider.insert(provider.to_string(), parsed_map);
            input_map.insert(provider.to_string(), all_inputs);
            supported_languages.insert(provider.to_string(), supported);
        }

        Ok(Self {
            by_provider,
            input_map,
            supported_languages,
        })
    }

    fn global() -> Result<&'static Self> {
        let maps = LANGUAGE_MAPS.get_or_init(Self::load);
        maps.as_ref()
            .map_err(|e| anyhow::anyhow!("Language maps not loaded: {}", e))
    }

    fn normalize(&self, provider: ProviderType, input: &str) -> Result<String> {
        let map = self.input_map(provider)?;
        Ok(Self::map_language_with_input_map(input, map))
    }

    fn input_map(&self, provider: ProviderType) -> Result<&ProviderInputMap> {
        self.input_map
            .get(provider.to_string().as_str())
            .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {provider}"))
    }

    fn raw_map(&self, provider: ProviderType) -> Result<&ProviderLanguageMap> {
        self.by_provider
            .get(provider.to_string().as_str())
            .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {provider}"))
    }

    fn supported_languages(&self, provider: ProviderType) -> Result<Vec<String>> {
        self.supported_languages
            .get(provider.to_string().as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {provider}"))
    }

    fn display_map(&self, provider: ProviderType) -> Result<HashMap<String, Vec<String>>> {
        let map = self.raw_map(provider)?;
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
        from_str(json).context("Failed to parse language map")
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

pub fn supported_languages_for_provider(provider: ProviderType) -> Result<Vec<String>> {
    LanguageMaps::global()?.supported_languages(provider)
}

pub fn map_language_for_provider(provider: ProviderType, input: &str) -> Result<String> {
    LanguageMaps::global()?.normalize(provider, input)
}

pub fn language_map_for_display(provider: ProviderType) -> Result<HashMap<String, Vec<String>>> {
    LanguageMaps::global()?.display_map(provider)
}
