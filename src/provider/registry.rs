use crate::provider::dev_templates::DevTemplatesProvider;
use crate::provider::devenv::DevenvProvider;
use crate::provider::language_maps::{
    language_map_for_display, map_language_for_provider, supported_languages_for_provider,
};
use anyhow::Result;

use super::{ProviderType, ShellProvider};

pub struct ProviderInfo {
    name: &'static str,
}

const PROVIDERS: [ProviderInfo; 2] = [
    ProviderInfo::new("devenv"),
    ProviderInfo::new("dev-templates"),
];

impl ProviderInfo {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn supported_languages(&self) -> Result<Vec<String>> {
        supported_languages_for_provider(self.name)
    }

    pub fn normalize_language(&self, input: &str) -> Result<String> {
        map_language_for_provider(self.name, input)
    }

    pub fn display_language_map(&self) -> Result<std::collections::HashMap<String, Vec<String>>> {
        language_map_for_display(self.name)
    }
}

pub fn all_providers() -> &'static [ProviderInfo] {
    &PROVIDERS
}

pub fn provider_info(provider_type: ProviderType) -> &'static ProviderInfo {
    match provider_type {
        ProviderType::Devenv => &PROVIDERS[0],
        ProviderType::DevTemplates => &PROVIDERS[1],
    }
}

pub fn into_shell_provider(provider_type: ProviderType) -> Box<dyn ShellProvider> {
    match provider_type {
        ProviderType::Devenv => Box::new(DevenvProvider),
        ProviderType::DevTemplates => Box::new(DevTemplatesProvider),
    }
}

pub fn provider_name(provider_type: ProviderType) -> &'static str {
    provider_info(provider_type).name()
}
