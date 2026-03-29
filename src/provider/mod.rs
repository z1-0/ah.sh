use anyhow::{Context, Result};
use serde_json::from_str;
use std::{collections::HashMap, sync::OnceLock};

pub mod dev_templates;
pub mod devenv;
pub mod language;

pub use language::{
    is_maybe_language, language_map_for_display, map_language_for_provider,
    normalize_and_dedup_languages, validate_supported_languages,
};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    clap::ValueEnum,
    serde::Deserialize,
    serde::Serialize,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    Devenv,
    DevTemplates,
}

/// Target of `ah provider show`: select a provider, or choose all.
#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderShowSelector {
    Devenv,
    DevTemplates,
    All,
}

struct Provider {
    pub supported_languages: Vec<String>,
    pub language_to_aliases: HashMap<String, Vec<String>>,
    pub alias_to_language: HashMap<String, String>,
}

impl ProviderShowSelector {
    pub fn as_provider_types(&self) -> &'static [ProviderType] {
        match self {
            ProviderShowSelector::Devenv => &[ProviderType::Devenv],
            ProviderShowSelector::DevTemplates => &[ProviderType::DevTemplates],
            ProviderShowSelector::All => &[ProviderType::Devenv, ProviderType::DevTemplates],
        }
    }
}

static PROVIDER_DEVENV: OnceLock<Result<Provider>> = OnceLock::new();
static PROVIDER_DEV_TEMPLATES: OnceLock<Result<Provider>> = OnceLock::new();

fn get_provider_once_lock(provider: ProviderType) -> &'static OnceLock<Result<Provider>> {
    match provider {
        ProviderType::Devenv => &PROVIDER_DEVENV,
        ProviderType::DevTemplates => &PROVIDER_DEV_TEMPLATES,
    }
}

pub fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
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
