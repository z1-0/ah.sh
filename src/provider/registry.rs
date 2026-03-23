use crate::provider::dev_templates::DevTemplatesProvider;
use crate::provider::devenv::DevenvProvider;
use crate::provider::language_maps::{
    language_map_for_display, map_language_for_provider, supported_languages_for_provider,
};
use anyhow::Result;

use super::{ProviderType, ShellProvider};

const PROVIDERS: [ProviderType; 2] = [ProviderType::Devenv, ProviderType::DevTemplates];

pub fn all_provider_types() -> &'static [ProviderType] {
    &PROVIDERS
}

pub fn into_shell_provider(provider_type: ProviderType) -> Box<dyn ShellProvider> {
    match provider_type {
        ProviderType::Devenv => Box::new(DevenvProvider),
        ProviderType::DevTemplates => Box::new(DevTemplatesProvider),
    }
}

pub fn provider_name(provider_type: ProviderType) -> &'static str {
    match provider_type {
        ProviderType::Devenv => "devenv",
        ProviderType::DevTemplates => "dev-templates",
    }
}

pub fn supported_languages(provider_type: ProviderType) -> Result<Vec<String>> {
    supported_languages_for_provider(provider_name(provider_type))
}

pub fn normalize_language(provider_type: ProviderType, input: &str) -> Result<String> {
    map_language_for_provider(provider_name(provider_type), input)
}

pub fn provider_language_map_for_display(
    provider_type: ProviderType,
) -> Result<std::collections::HashMap<String, Vec<String>>> {
    language_map_for_display(provider_name(provider_type))
}
