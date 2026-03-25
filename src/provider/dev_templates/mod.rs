pub mod flake_generator;
pub mod nix_parser;

use crate::cmd;
use crate::provider::dev_templates::flake_generator::generate_dev_templates_flake;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;

/// Special language name representing an empty/languageless shell.
/// Matches the "empty" template in dev-templates.
const EMPTY_LANGUAGE: &str = "empty";

pub fn get_flake_contents(languages: &[String]) -> Result<String> {
    // Filter out empty language first, then deduplicate while preserving order.
    // Order preservation is critical for matching parsed_attrs index to language.
    let mut seen = HashSet::new();
    let deduped_languages: Vec<String> = languages
        .iter()
        .filter(|lang| *lang != EMPTY_LANGUAGE)
        .filter(|lang| seen.insert(lang.as_str()))
        .cloned()
        .collect();

    // Single prefetch for the main dev-templates repo
    let prefetch_raw = cmd::prefetch_dev_templates()?;
    let nix_store_path = get_nix_store_path(prefetch_raw)?;

    let parsed_attrs: Vec<ShellAttrs> = deduped_languages
        .par_iter()
        .map(|lang| parse_flake(&nix_store_path, lang))
        .collect::<Result<Vec<_>>>()?;

    let flake_content = generate_dev_templates_flake(&deduped_languages, &parsed_attrs);

    Ok(flake_content)
}

fn get_nix_store_path(prefetch_raw: String) -> Result<String> {
    let json: serde_json::Value =
        serde_json::from_str(&prefetch_raw).context("failed to parse prefetch response as JSON")?;

    let store_path = json
        .get("storePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing storePath in prefetch response"))?;

    Ok(store_path.to_string())
}

fn parse_flake(store_path: &str, language: &str) -> Result<ShellAttrs> {
    let flake_path = format!("{store_path}/{language}/flake.nix");

    let flake_contents = fs::read_to_string(&flake_path)
        .with_context(|| format!("failed to read {} for language '{}'", flake_path, language))?;

    Ok(nix_parser::parse_flake_shell(&flake_contents))
}
