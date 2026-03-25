use crate::provider::ProviderType;
use anyhow::{Context, Result};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::OnceLock;

// Type aliases for language mapping data
pub(crate) type InputMapping = HashMap<String, String>; // user_input -> canonical_name
pub(crate) type LanguageMappings = HashMap<String, Vec<String>>; // canonical_name -> [user_inputs]

// Internal storage indexed by provider name
type Mappings = HashMap<ProviderType, LanguageMappings>;
type Inputs = HashMap<ProviderType, InputMapping>;
type Supported = HashMap<ProviderType, Vec<String>>;

struct LanguageMaps {
    mappings: Mappings,   // canonical -> inputs per provider
    inputs: Inputs,       // input -> canonical per provider
    supported: Supported, // supported languages list per provider
}

static LANGUAGE_MAPS: OnceLock<Result<LanguageMaps>> = OnceLock::new();

const PROVIDER_CONFIGS: [(ProviderType, &str, &str); 2] = [
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

// ============================================================================
// Public API
// ============================================================================

pub fn is_maybe_language(input: &str) -> bool {
    LanguageMaps::global()
        .map(|maps| maps.contains_input(input))
        .unwrap_or(false)
}

pub fn get_supported_languages(provider: ProviderType) -> Result<Vec<String>> {
    LanguageMaps::global()?.get_supported_languages(provider)
}

pub fn map_language_for_provider(provider: ProviderType, input: &str) -> Result<String> {
    LanguageMaps::global()?.lookup_input(provider, input)
}

pub fn language_map_for_display(provider: ProviderType) -> Result<HashMap<String, Vec<String>>> {
    LanguageMaps::global()?.get_display_mappings(provider)
}

// ============================================================================
// LanguageMaps implementation
// ============================================================================

impl LanguageMaps {
    fn load() -> Result<Self> {
        let mut mappings = HashMap::new();
        let mut inputs = HashMap::new();
        let mut supported = HashMap::new();

        for (provider, supported_json, map_json) in PROVIDER_CONFIGS {
            let lang_mappings = from_str(map_json).context("Failed to parse language mappings")?;

            let supported_langs: Vec<String> =
                from_str(supported_json).context("Failed to parse supported_langs")?;

            let mut input_mapping = Self::build_input_mappings(&lang_mappings);
            for lang in &supported_langs {
                input_mapping
                    .entry(lang.clone())
                    .or_insert_with(|| lang.clone());
            }

            mappings.insert(provider, lang_mappings);
            inputs.insert(provider, input_mapping);
            supported.insert(provider, supported_langs);
        }

        Ok(Self {
            mappings,
            inputs,
            supported,
        })
    }

    fn global() -> Result<&'static Self> {
        let maps = LANGUAGE_MAPS.get_or_init(Self::load);
        maps.as_ref()
            .map_err(|e| anyhow::anyhow!("Language maps not loaded: {}", e))
    }

    fn lookup_input(&self, provider: ProviderType, input: &str) -> Result<String> {
        Ok(self
            .get_from_map(&self.inputs, provider)?
            .get(input)
            .cloned()
            .unwrap_or_else(|| input.to_string()))
    }

    fn get_display_mappings(&self, provider: ProviderType) -> Result<HashMap<String, Vec<String>>> {
        Ok(Self::format_display_variants(
            self.get_from_map(&self.mappings, provider)?,
        ))
    }

    fn contains_input(&self, input: &str) -> bool {
        self.inputs
            .values()
            .any(|mapping| mapping.contains_key(input))
    }

    // Generic helper to eliminate duplicate lookup patterns
    fn get_from_map<'a, T>(
        &self,
        map: &'a HashMap<ProviderType, T>,
        provider: ProviderType,
    ) -> Result<&'a T> {
        map.get(&provider)
            .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {provider}"))
    }

    fn get_supported_languages(&self, provider: ProviderType) -> Result<Vec<String>> {
        self.get_from_map(&self.supported, provider).cloned()
    }

    fn format_display_variants(mappings: &LanguageMappings) -> HashMap<String, Vec<String>> {
        let mut display = HashMap::new();

        for (canonical, variants) in mappings {
            let mut display_variants: Vec<String> = variants
                .iter()
                .filter(|v| v != &canonical)
                .cloned()
                .collect();
            display_variants.sort();
            display_variants.dedup();
            display.insert(canonical.clone(), display_variants);
        }

        display
    }

    fn build_input_mappings(mappings: &LanguageMappings) -> InputMapping {
        let mut inputs = InputMapping::new();

        for (canonical, variants) in mappings {
            for variant in variants {
                inputs.insert(variant.clone(), canonical.clone());
            }
        }

        inputs
    }
}
