use crate::provider::{Language, ProviderType, Supported, get_or_init_provider};
use anyhow::Result;
use std::collections::HashMap;

pub fn is_maybe_language(provider: ProviderType, language: &str) -> Result<bool> {
    Ok(get_or_init_provider(provider)?
        .alias_to_language
        .contains_key(language))
}

pub fn get_supported_languages(provider: ProviderType) -> Result<&'static [String]> {
    Ok(get_or_init_provider(provider)?
        .supported_languages
        .as_slice())
}

pub fn language_map_for_display(
    provider: ProviderType,
) -> Result<&'static HashMap<String, Vec<String>>> {
    Ok(&get_or_init_provider(provider)?.language_to_aliases)
}

pub fn to_supported_languages(
    provider: ProviderType,
    languages: &[Language],
) -> Result<Vec<Supported>> {
    let alias_to_language = &get_or_init_provider(provider)?.alias_to_language;
    let mut supported_languages = Vec::with_capacity(languages.len());
    let mut unsupported_languages = Vec::new();

    for language in languages {
        if let Some(supported) = alias_to_language.get(language) {
            supported_languages.push(supported.clone());
        } else {
            unsupported_languages.push(language.clone());
        }
    }

    if !unsupported_languages.is_empty() {
        anyhow::bail!("unsupported languages: {:?}", unsupported_languages);
    }

    supported_languages.sort_unstable();
    supported_languages.dedup();
    Ok(supported_languages)
}
