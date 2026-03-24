use crate::provider::dev_templates;
use crate::provider::devenv;
use crate::provider::language_maps::{
    language_map_for_display, map_language_for_provider, supported_languages_for_provider,
};
use anyhow::Result;

use super::ProviderType;

const PROVIDERS: [ProviderType; 2] = [ProviderType::Devenv, ProviderType::DevTemplates];

pub fn all_provider_types() -> &'static [ProviderType] {
    &PROVIDERS
}

pub(crate) fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
    }
}

pub fn supported_languages(provider: ProviderType) -> Result<Vec<String>> {
    supported_languages_for_provider(provider)
}

pub fn normalize_language(provider: ProviderType, input: &str) -> Result<String> {
    map_language_for_provider(provider, input)
}

pub fn provider_language_map_for_display(
    provider: ProviderType,
) -> Result<std::collections::HashMap<String, Vec<String>>> {
    language_map_for_display(provider)
}
