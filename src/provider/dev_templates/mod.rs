pub mod flake_generator;
pub mod nix_parser;

use crate::cmd;
use crate::error::{AppError, Result};
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::provider::{EnsureFilesResult, ShellProvider};
use crate::warning::AppWarning;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
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

    // Single prefetch for the main dev-templates repo
    let store_path = prefetch_dev_templates()?;

    let results: Vec<LanguageOutcome> = deduped_languages
        .par_iter()
        .filter(|lang| **lang != EMPTY_LANGUAGE)
        .map(|lang| resolve_language(&store_path, lang))
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

fn prefetch_dev_templates() -> Result<String> {
    let prefetch_raw = cmd::nix_flake_prefetch_dev_templates()?;
    serde_json::from_str::<serde_json::Value>(&prefetch_raw)
        .ok()
        .and_then(|mut v| v["storePath"].take().as_str().map(String::from))
        .ok_or_else(|| AppError::Provider("missing storePath in prefetch response".into()))
}

/// Read the flake.nix file for a language from the Nix store.
fn read_flake_file(store_path: &str, lang: &str) -> Result<String> {
    let flake_path = format!("{store_path}/{lang}/flake.nix");
    fs::read_to_string(&flake_path)
        .map_err(|err| AppError::Provider(format!("failed to read {flake_path}: {err}")))
}

struct LanguageOutcome {
    language: String,
    attrs: Option<ShellAttrs>,
    warnings: Vec<AppWarning>,
}

fn resolve_language(store_path: &str, language: &str) -> LanguageOutcome {
    let mut warnings = Vec::new();

    let flake_source = match read_flake_file(store_path, language) {
        Ok(source) => source,
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

    let attrs: ShellAttrs = nix_parser::parse_flake_shell(&flake_source);

    LanguageOutcome {
        language: language.to_string(),
        attrs: Some(attrs),
        warnings,
    }
}
