use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Context, Result};
use serde_json::from_str;
use tracing_attributes::instrument;

pub mod dev_templates;
pub mod devenv;
pub mod types;

pub use types::*;

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

#[instrument(skip_all)]
fn init_provider(provider: ProviderType) -> Result<Provider> {
    let supported_languages: Vec<Supported> = from_str(supported_languages_json(provider))
        .with_context(|| format!("Failed to parse supported languages for {provider}"))?;

    let language_to_aliases: HashMap<Supported, Vec<Alias>> =
        from_str(language_aliases_json(provider))
            .with_context(|| format!("Failed to parse language mappings for {provider}"))?;

    let capacity =
        supported_languages.len() + language_to_aliases.values().map(Vec::len).sum::<usize>();
    let mut alias_to_language = HashMap::with_capacity(capacity);
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

#[instrument(skip_all)]
pub fn get_provider(provider: ProviderType) -> &'static Provider {
    static PROVIDER_DEVENV: LazyLock<Provider> = LazyLock::new(|| {
        init_provider(ProviderType::Devenv).expect("Failed to initialize devenv provider")
    });
    static PROVIDER_DEV_TEMPLATES: LazyLock<Provider> = LazyLock::new(|| {
        init_provider(ProviderType::DevTemplates)
            .expect("Failed to initialize dev-templates provider")
    });

    match provider {
        ProviderType::Devenv => &PROVIDER_DEVENV,
        ProviderType::DevTemplates => &PROVIDER_DEV_TEMPLATES,
    }
}

#[instrument(skip_all)]
pub fn to_supported_languages(
    provider: ProviderType,
    languages: &[Language],
) -> Result<Vec<Supported>> {
    let alias_to_language = get_provider(provider).get_alias_to_language();
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

#[instrument(skip_all)]
pub fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
    }
}
