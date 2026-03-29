use crate::provider::ProviderType;
use anyhow::{Context, Result};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Provider {
    pub supported_languages: Vec<String>,
    pub language_to_aliases: HashMap<String, Vec<String>>,
    pub alias_to_language: HashMap<String, String>,
}

static DEVENV_PROVIDER: OnceLock<Result<Provider>> = OnceLock::new();
static DEV_TEMPLATES_PROVIDER: OnceLock<Result<Provider>> = OnceLock::new();

fn get_provider(provider: ProviderType) -> &'static OnceLock<Result<Provider>> {
    match provider {
        ProviderType::Devenv => &DEVENV_PROVIDER,
        ProviderType::DevTemplates => &DEV_TEMPLATES_PROVIDER,
    }
}

fn supported_languages_json(provider: ProviderType) -> &'static str {
    match provider {
        ProviderType::Devenv => {
            include_str!("../assets/providers/devenv/supported_languages.json")
        }
        ProviderType::DevTemplates => {
            include_str!("../assets/providers/dev-templates/supported_languages.json")
        }
    }
}

fn language_aliases_json(provider: ProviderType) -> &'static str {
    match provider {
        ProviderType::Devenv => {
            include_str!("../assets/providers/devenv/language_aliases.json")
        }
        ProviderType::DevTemplates => {
            include_str!("../assets/providers/dev-templates/language_aliases.json")
        }
    }
}

fn get_provider_map(provider: ProviderType) -> Result<&'static Provider> {
    get_provider(provider)
        .get_or_init(|| load(provider))
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Language map not loaded for {provider}: {e}"))
}

pub fn is_maybe_language(provider: ProviderType, input: &str) -> Result<bool> {
    Ok(get_provider_map(provider)?
        .alias_to_language
        .contains_key(input))
}

pub fn get_supported_languages(provider: ProviderType) -> Result<&'static [String]> {
    Ok(get_provider_map(provider)?.supported_languages.as_slice())
}

pub fn map_language_for_provider(provider: ProviderType, input: &str) -> Result<String> {
    Ok(get_provider_map(provider)?
        .alias_to_language
        .get(input)
        .cloned()
        .unwrap_or_else(|| input.to_string()))
}

pub fn language_map_for_display(
    provider: ProviderType,
) -> Result<&'static HashMap<String, Vec<String>>> {
    Ok(&get_provider_map(provider)?.language_to_aliases)
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

fn build_alias_to_language(
    language_to_aliases: &HashMap<String, Vec<String>>,
    supported_languages: &[String],
) -> HashMap<String, String> {
    let mut alias_to_language = HashMap::new();

    for language in supported_languages {
        alias_to_language.insert(language.clone(), language.clone());
    }

    for (language, aliases) in language_to_aliases {
        for alias in aliases {
            alias_to_language.insert(alias.clone(), language.clone());
        }
    }

    alias_to_language
}

fn load(provider: ProviderType) -> Result<Provider> {
    let supported_languages: Vec<String> = from_str(supported_languages_json(provider))
        .with_context(|| format!("Failed to parse supported languages for {provider}"))?;

    let language_to_aliases: HashMap<String, Vec<String>> =
        from_str(language_aliases_json(provider))
            .with_context(|| format!("Failed to parse language mappings for {provider}"))?;

    let alias_to_language = build_alias_to_language(&language_to_aliases, &supported_languages);

    Ok(Provider {
        supported_languages,
        language_to_aliases,
        alias_to_language,
    })
}
