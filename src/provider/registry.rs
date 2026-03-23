use crate::provider::dev_templates;
use crate::provider::devenv;
use crate::provider::language_maps::{
    language_map_for_display, map_language_for_provider, supported_languages_for_provider,
};
use anyhow::Result;
use std::path::Path;

use super::ProviderType;

const PROVIDERS: [ProviderType; 2] = [ProviderType::Devenv, ProviderType::DevTemplates];

type EnsureFilesFn = fn(&[String], &Path) -> Result<()>;

pub fn all_provider_types() -> &'static [ProviderType] {
    &PROVIDERS
}

pub fn ensure_files(
    provider_type: ProviderType,
    languages: &[String],
    target_dir: &Path,
) -> Result<()> {
    provider_ensure_files(provider_type)(languages, target_dir)
}

fn provider_ensure_files(provider_type: ProviderType) -> EnsureFilesFn {
    match provider_type {
        ProviderType::Devenv => devenv::ensure_files,
        ProviderType::DevTemplates => dev_templates::ensure_files,
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
