use crate::cmd;
use crate::error::{AppError, Result};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct PrefetchResponse {
    #[serde(rename = "storePath")]
    store_path: String,
}

pub fn prefetch_dev_templates() -> Result<String> {
    let prefetch_raw = cmd::nix_flake_prefetch_dev_templates()?;
    let prefetch: PrefetchResponse = serde_json::from_str(&prefetch_raw)?;
    Ok(prefetch.store_path)
}

pub fn resolve_language(store_path: &str, lang: &str) -> Result<String> {
    let flake_path = format!("{}/{}/flake.nix", store_path, lang);
    fs::read_to_string(&flake_path)
        .map_err(|err| AppError::Provider(format!("failed to read {flake_path}: {err}")))
}
