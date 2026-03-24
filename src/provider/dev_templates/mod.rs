pub mod flake_generator;
pub mod nix_parser;

use crate::cmd;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;

const EMPTY_LANGUAGE: &str = "empty";

pub fn get_flake_contents(languages: &[String]) -> Result<String> {
    let mut seen = HashSet::new();
    let deduped_languages: Vec<String> = languages
        .iter()
        .filter(|lang| seen.insert(*lang))
        .cloned()
        .collect();

    // Single prefetch for the main dev-templates repo
    let prefetch_raw = cmd::prefetch_dev_templates()?;
    let nix_store_path = get_nix_store_path(prefetch_raw)?;

    let results: Vec<LanguageOutcome> = deduped_languages
        .par_iter()
        .filter(|lang| **lang != EMPTY_LANGUAGE)
        .map(|lang| resolve_language(&nix_store_path, lang))
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

    Ok(flake_content)
}

fn get_nix_store_path(prefetch_raw: String) -> Result<String> {
    serde_json::from_str::<serde_json::Value>(&prefetch_raw)
        .ok()
        .and_then(|mut v| v["storePath"].take().as_str().map(String::from))
        .ok_or_else(|| anyhow::anyhow!("missing storePath in prefetch response"))
}

/// Read the flake.nix file for a language from the Nix store.
fn read_flake_file(store_path: &str, lang: &str) -> Result<String> {
    let flake_path = format!("{store_path}/{lang}/flake.nix");
    fs::read_to_string(&flake_path).context(format!("failed to read {flake_path}"))
}

struct LanguageOutcome {
    language: String,
    attrs: Option<ShellAttrs>,
    warnings: Vec<String>,
}

fn resolve_language(store_path: &str, language: &str) -> LanguageOutcome {
    let mut warnings = Vec::new();

    let flake_source = match read_flake_file(store_path, language) {
        Ok(source) => source,
        Err(err) => {
            let warning_msg = format!("failed to resolve language {}: {}", language, err);
            tracing::warn!(language = %language, error = %err, "dev_templates.resolve_failed");
            warnings.push(warning_msg);

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
