use crate::provider::{ProviderType, get_or_init_provider};
use anyhow::Result;
use std::collections::HashMap;

pub fn is_maybe_language(provider: ProviderType, input: &str) -> Result<bool> {
    Ok(get_or_init_provider(provider)?
        .alias_to_language
        .contains_key(input))
}

pub fn get_supported_languages(provider: ProviderType) -> Result<&'static [String]> {
    Ok(get_or_init_provider(provider)?
        .supported_languages
        .as_slice())
}

pub fn map_language_for_provider(provider: ProviderType, input: &str) -> Result<String> {
    Ok(get_or_init_provider(provider)?
        .alias_to_language
        .get(input)
        .cloned()
        .unwrap_or_else(|| input.to_string()))
}

pub fn language_map_for_display(
    provider: ProviderType,
) -> Result<&'static HashMap<String, Vec<String>>> {
    Ok(&get_or_init_provider(provider)?.language_to_aliases)
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

pub fn validate_supported_languages(provider: ProviderType, languages: &[String]) -> Result<()> {
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
