pub mod dev_templates;
pub mod devenv;
pub mod types;

use anyhow::{Context, Result};
use serde_json::from_str;
use std::{collections::HashMap, sync::OnceLock};

pub use types::*;

static PROVIDER_DEVENV: OnceLock<Result<Provider>> = OnceLock::new();
static PROVIDER_DEV_TEMPLATES: OnceLock<Result<Provider>> = OnceLock::new();

fn get_provider_once_lock(provider: ProviderType) -> &'static OnceLock<Result<Provider>> {
    match provider {
        ProviderType::Devenv => &PROVIDER_DEVENV,
        ProviderType::DevTemplates => &PROVIDER_DEV_TEMPLATES,
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

fn get_or_init_provider(provider: ProviderType) -> Result<&'static Provider> {
    get_provider_once_lock(provider)
        .get_or_init(|| init_provider(provider))
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Language map not loaded for {provider}: {e}"))
}

fn init_provider(provider: ProviderType) -> Result<Provider> {
    let supported_languages: Vec<Supported> = from_str(supported_languages_json(provider))
        .with_context(|| format!("Failed to parse supported languages for {provider}"))?;

    let language_to_aliases: HashMap<Supported, Vec<Alias>> =
        from_str(language_aliases_json(provider))
            .with_context(|| format!("Failed to parse language mappings for {provider}"))?;

    let mut alias_to_language = HashMap::new();
    for lang in &supported_languages {
        alias_to_language.insert(lang.clone(), lang.clone());
    }
    for (lang, aliases) in &language_to_aliases {
        for alias in aliases {
            alias_to_language.insert(alias.clone(), lang.clone());
        }
    }

    Ok(Provider::new(
        supported_languages,
        language_to_aliases,
        alias_to_language,
    ))
}

pub fn to_supported_languages(
    provider: ProviderType,
    languages: &[Language],
) -> Result<Vec<Supported>> {
    let alias_to_language = provider.to_provider()?.get_alias_to_language();
    let mut supported_languages = Vec::with_capacity(languages.len());
    let mut unsupported_languages = Vec::new();

    for language in languages {
        if let Some(supported) = alias_to_language.get(language) {
            supported_languages.push(supported.clone());
        } else {
            unsupported_languages.push(language.clone());
        }
    }

    if !unsupported_languages.is_empty() {
        anyhow::bail!("unsupported languages: {:?}", unsupported_languages);
    }

    supported_languages.sort_unstable();
    supported_languages.dedup();
    Ok(supported_languages)
}

pub fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
    }
}
