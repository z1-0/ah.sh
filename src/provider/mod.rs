pub mod dev_templates;
pub mod devenv;
pub mod language;

use anyhow::Result;

pub use language::{
    is_maybe_language, language_map_for_display, map_language_for_provider,
    normalize_and_dedup_languages, validate_supported_languages,
};

#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    strum::Display,
    strum::EnumString,
    Eq,
    PartialEq,
    clap::ValueEnum,
    serde::Deserialize,
    serde::Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
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

impl ProviderShowSelector {
    pub fn as_provider_types(&self) -> &'static [ProviderType] {
        match self {
            ProviderShowSelector::Devenv => &[ProviderType::Devenv],
            ProviderShowSelector::DevTemplates => &[ProviderType::DevTemplates],
            ProviderShowSelector::All => &[ProviderType::Devenv, ProviderType::DevTemplates],
        }
    }
}

pub fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
    }
}
