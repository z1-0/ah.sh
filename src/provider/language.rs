use crate::provider::ProviderType;
use anyhow::{Context, Result};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct LanguageCache {
    pub inputs: HashMap<String, String>, // user_input -> canonical
    pub supported: Vec<String>,          // supported canonical languages
    pub display: HashMap<String, Vec<String>>, // canonical -> sorted unique variants for display
}

static DEVENV_LANGUAGE_MAP: OnceLock<Result<LanguageCache>> = OnceLock::new();
static DEV_TEMPLATES_LANGUAGE_MAP: OnceLock<Result<LanguageCache>> = OnceLock::new();

impl ProviderType {
    pub(crate) fn cache_cell(&self) -> &'static OnceLock<Result<LanguageCache>> {
        match self {
            ProviderType::Devenv => &DEVENV_LANGUAGE_MAP,
            ProviderType::DevTemplates => &DEV_TEMPLATES_LANGUAGE_MAP,
        }
    }

    pub(crate) fn supported_languages(&self) -> &'static str {
        match self {
            ProviderType::Devenv => {
                include_str!("../assets/providers/devenv/supported_languages.json")
            }
            ProviderType::DevTemplates => {
                include_str!("../assets/providers/dev-templates/supported_languages.json")
            }
        }
    }

    pub(crate) fn language_aliases(&self) -> &'static str {
        match self {
            ProviderType::Devenv => {
                include_str!("../assets/providers/devenv/language_aliases.json")
            }
            ProviderType::DevTemplates => {
                include_str!("../assets/providers/dev-templates/language_aliases.json")
            }
        }
    }
}

fn get_provider_map(provider: ProviderType) -> Result<&'static LanguageCache> {
    provider
        .cache_cell()
        .get_or_init(|| LanguageCache::load(provider))
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Language map not loaded for {provider}: {e}"))
}

pub fn is_maybe_language(provider: ProviderType, input: &str) -> Result<bool> {
    Ok(get_provider_map(provider)?.inputs.contains_key(input))
}

pub fn get_supported_languages(provider: ProviderType) -> Result<&'static [String]> {
    Ok(get_provider_map(provider)?.supported.as_slice())
}

pub fn map_language_for_provider(provider: ProviderType, input: &str) -> Result<String> {
    Ok(get_provider_map(provider)?
        .inputs
        .get(input)
        .cloned()
        .unwrap_or_else(|| input.to_string()))
}

pub fn language_map_for_display(
    provider: ProviderType,
) -> Result<&'static HashMap<String, Vec<String>>> {
    Ok(&get_provider_map(provider)?.display)
}

pub fn normalize_and_dedup_languages(
    provider: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    let mut mapped_langs = languages
        .iter()
        .map(|language| map_language_for_provider(provider, language))
        .collect::<Result<Vec<_>>>()?;

    mapped_langs.sort_unstable();
    mapped_langs.dedup();
    Ok(mapped_langs)
}

pub fn validate_supported_languages(provider: ProviderType, languages: &[String]) -> Result<()> {
    let supported = get_supported_languages(provider)?;
    let supported_set: std::collections::HashSet<_> = supported.iter().collect();
    let invalids: Vec<_> = languages
        .iter()
        .filter(|language| !supported_set.contains(language))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("unsupported languages: {:?}", invalids)
    }
}

impl LanguageCache {
    fn load(provider: ProviderType) -> Result<Self> {
        let supported_languages: Vec<String> = from_str(provider.supported_languages())
            .with_context(|| format!("Failed to parse supported languages for {provider}"))?;

        let language_to_aliases: HashMap<String, Vec<String>> =
            from_str(provider.language_aliases())
                .with_context(|| format!("Failed to parse language mappings for {provider}"))?;

        let inputs = Self::build_input_mappings(&language_to_aliases, &supported_languages);
        let display = Self::format_display_variants(&language_to_aliases);

        Ok(Self {
            inputs,
            supported: supported_languages,
            display,
        })
    }

    fn build_input_mappings(
        language_to_aliases: &HashMap<String, Vec<String>>,
        supported: &[String],
    ) -> HashMap<String, String> {
        let inputs = language_to_aliases
            .iter()
            .fold(HashMap::new(), |mut acc, (lang, aliases)| {
                acc.entry(lang.clone()).or_insert_with(|| lang.clone());
                Self::normalized_aliases(lang, aliases)
                    .into_iter()
                    .for_each(|alias| {
                        acc.insert(alias.clone(), lang.clone());
                    });

                acc
            });

        supported.iter().fold(inputs, |mut acc, lang| {
            acc.entry(lang.clone()).or_insert_with(|| lang.clone());
            acc
        })
    }

    fn format_display_variants(
        language_to_aliases: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<String>> {
        language_to_aliases
            .iter()
            .map(|(lang, aliases)| (lang.clone(), Self::normalized_aliases(lang, aliases)))
            .collect()
    }

    fn normalized_aliases(lang: &str, aliases: &[String]) -> Vec<String> {
        let mut normalized: Vec<String> = aliases
            .iter()
            .filter(|alias| alias.as_str() != lang)
            .cloned()
            .collect();
        normalized.sort();
        normalized.dedup();
        normalized
    }
}
