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
pub enum ProviderKeyOrAll {
    Devenv,
    DevTemplates,
    All,
}

impl ProviderKeyOrAll {
    pub fn as_provider_types(&self) -> &'static [ProviderType] {
        match self {
            ProviderKeyOrAll::Devenv => &[ProviderType::Devenv],
            ProviderKeyOrAll::DevTemplates => &[ProviderType::DevTemplates],
            ProviderKeyOrAll::All => &[ProviderType::Devenv, ProviderType::DevTemplates],
        }
    }
}
