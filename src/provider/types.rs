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
    schemars::JsonSchema,
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
