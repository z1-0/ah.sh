pub mod attrs_cache;
pub mod flake_generator;
pub mod nix_parser;
pub mod store_resolver;

use crate::error::Result;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::provider::{EnsureFilesResult, ShellProvider};
use crate::warning::AppWarning;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;

const EMPTY_LANGUAGE: &str = "empty";

pub struct DevTemplatesProvider;

impl ShellProvider for DevTemplatesProvider {
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult> {
        ensure_files_impl(languages, target_dir)
    }
}

fn ensure_files_impl(languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult> {
    let mut seen = HashSet::new();
    let deduped_languages: Vec<String> = languages
        .iter()
        .filter(|lang| seen.insert(*lang))
        .cloned()
        .collect();

    let results: Vec<LanguageOutcome> = deduped_languages
        .par_iter()
        .filter(|lang| **lang != EMPTY_LANGUAGE)
        .map(|lang| resolve_language(lang))
        .collect();

    let mut parsed_attrs = Vec::new();
    let mut warnings = Vec::new();

    for outcome in results {
        if let Some(attrs) = outcome.attrs {
            parsed_attrs.push((outcome.language, attrs));
        }
        warnings.extend(outcome.warnings);
    }

    let flake_content =
        flake_generator::generate_dev_templates_flake(&deduped_languages, &parsed_attrs);
    std::fs::write(target_dir.join("flake.nix"), flake_content)?;

    Ok(EnsureFilesResult { warnings })
}

struct LanguageOutcome {
    language: String,
    attrs: Option<ShellAttrs>,
    warnings: Vec<AppWarning>,
}

fn resolve_language(language: &str) -> LanguageOutcome {
    let mut warnings = Vec::new();

    let resolved = match store_resolver::resolve_store_source(language) {
        Ok(resolved) => resolved,
        Err(err) => {
            warnings.push(
                AppWarning::new("dev_templates.resolve_failed", err.to_string())
                    .with_context("language", language.to_string()),
            );

            return LanguageOutcome {
                language: language.to_string(),
                attrs: None,
                warnings,
            };
        }
    };

    match attrs_cache::load_cached_attrs(language, &resolved.locked_key) {
        Ok((Some(attrs), warning)) => {
            warnings.extend(warning);
            return LanguageOutcome {
                language: language.to_string(),
                attrs: Some(attrs),
                warnings,
            };
        }
        Ok((None, warning)) => warnings.extend(warning),
        Err(err) => {
            warnings.push(
                AppWarning::new("dev_templates.attrs_cache_load_failed", err.to_string())
                    .with_context("language", language.to_string()),
            );
        }
    }

    let attrs = nix_parser::parse_flake_shell(&resolved.flake_source);

    if let Some(warning) = attrs_cache::save_cached_attrs(language, &resolved.locked_key, &attrs) {
        warnings.push(warning);
    }

    LanguageOutcome {
        language: language.to_string(),
        attrs: Some(attrs),
        warnings,
    }
}
