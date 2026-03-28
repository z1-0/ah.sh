use crate::provider::dev_templates;
use crate::provider::devenv;
use crate::provider::language::{
    language_map_for_display, normalize_and_dedup_languages as normalize_languages,
    validate_supported_languages,
};
use anyhow::Result;

use super::ProviderType;

pub(crate) fn get_flake_contents(provider: ProviderType) -> fn(&[String]) -> Result<String> {
    match provider {
        ProviderType::Devenv => devenv::get_flake_contents,
        ProviderType::DevTemplates => dev_templates::get_flake_contents,
    }
}

pub fn provider_language_map_for_display(
    provider: ProviderType,
) -> Result<&'static std::collections::HashMap<String, Vec<String>>> {
    language_map_for_display(provider)
}

pub fn validate_languages(provider: ProviderType, languages: &[String]) -> Result<()> {
    validate_supported_languages(provider, languages)
}

pub fn normalize_and_dedup_languages(
    provider: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    normalize_languages(provider, languages)
}
