use std::collections::HashMap;

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

impl ProviderType {
    pub fn to_provider(self) -> anyhow::Result<&'static super::Provider> {
        super::get_or_init_provider(self)
    }
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

pub type Language = String;
pub type Supported = String;
pub type Alias = String;

pub struct Provider {
    supported_languages: Vec<Supported>,
    language_to_aliases: HashMap<Supported, Vec<Alias>>,
    alias_to_language: HashMap<Alias, Supported>,
}

impl Provider {
    pub fn new(
        supported_languages: Vec<Supported>,
        language_to_aliases: HashMap<Supported, Vec<Alias>>,
        alias_to_language: HashMap<Alias, Supported>,
    ) -> Self {
        Self {
            supported_languages,
            language_to_aliases,
            alias_to_language,
        }
    }

    pub fn get_supported_languages(&self) -> &[Supported] {
        &self.supported_languages
    }

    pub fn get_language_to_aliases(&self) -> &HashMap<Supported, Vec<Alias>> {
        &self.language_to_aliases
    }

    pub fn get_alias_to_language(&self) -> &HashMap<Alias, Supported> {
        &self.alias_to_language
    }
}
