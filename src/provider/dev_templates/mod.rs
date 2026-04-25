pub mod flake_generator;
pub mod nix_parser;

use crate::cmd;
use crate::provider::dev_templates::flake_generator::generate_dev_templates_flake;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use anyhow::Result;
use fs_err as fs;
use rayon::prelude::*;
use std::collections::HashSet;
use tracing_attributes::instrument;

const EMPTY_LANGUAGE: &str = "empty";

#[instrument(skip_all, err)]
pub fn get_flake_contents(languages: &[String]) -> Result<String> {
    let mut seen = HashSet::new();
    let deduped_languages: Vec<String> = languages
        .iter()
        .filter(|lang| *lang != EMPTY_LANGUAGE)
        .filter(|lang| seen.insert(lang.as_str()))
        .cloned()
        .collect();

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
    let json: serde_json::Value = serde_json::from_str(&prefetch_raw)?;

    let store_path = json
        .get("storePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing storePath in prefetch response"))?;

    Ok(store_path.to_string())
}

#[instrument(skip_all, fields(language))]
fn parse_flake(store_path: &str, language: &str) -> Result<ShellAttrs> {
    let flake_path = format!("{store_path}/{language}/flake.nix");

    let flake_contents = fs::read_to_string(flake_path)?;

    Ok(nix_parser::parse_flake_shell(&flake_contents))
}
