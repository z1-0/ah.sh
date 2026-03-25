use crate::provider::dev_templates;
use crate::provider::devenv;
use crate::provider::language_maps::{
    get_supported_languages, language_map_for_display, map_language_for_provider,
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

pub fn provider_language_map_for_display(
    provider: ProviderType,
) -> Result<std::collections::HashMap<String, Vec<String>>> {
    language_map_for_display(provider)
}

pub fn validate_languages(provider: ProviderType, languages: &[String]) -> Result<()> {
    let supported = get_supported_languages(provider)?;
    let supported_set: std::collections::HashSet<_> = supported.iter().collect();
    let invalids: Vec<_> = languages
        .iter()
        .filter(|language| !supported_set.contains(language))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("unsupported languages: {:?}", invalids)
    }
}

pub fn normalize_and_dedup_languages(
    provider: ProviderType,
    languages: &[String],
) -> Result<Vec<String>> {
    let mut mapped_langs = languages
        .iter()
        .map(|language| map_language_for_provider(provider, language))
        .collect::<Result<Vec<_>>>()?;

    mapped_langs.sort_unstable();
    mapped_langs.dedup();

    Ok(mapped_langs)
}
