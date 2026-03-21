use crate::cmd;
use crate::error::{AppError, Result};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStoreSource {
    pub locked_key: String,
    pub flake_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct PrefetchResponse {
    hash: String,
    locked: PrefetchLocked,
    original: PrefetchOriginal,
    #[serde(rename = "storePath")]
    store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct PrefetchLocked {
    #[serde(rename = "lastModified")]
    last_modified: i64,
    owner: String,
    repo: String,
    rev: String,
    #[serde(rename = "type")]
    source_type: String,
    dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct PrefetchOriginal {
    owner: String,
    repo: String,
    #[serde(rename = "type")]
    source_type: String,
    dir: Option<String>,
}

pub fn resolve_store_source(lang: &str) -> Result<ResolvedStoreSource> {
    let prefetch_raw = cmd::nix_flake_prefetch(lang)?;
    let prefetch: PrefetchResponse = serde_json::from_str(&prefetch_raw)?;

    let flake_path = format!("{}/{}/flake.nix", prefetch.store_path, lang);
    let flake_source = fs::read_to_string(&flake_path)
        .map_err(|err| AppError::Provider(format!("failed to read {flake_path}: {err}")))?;

    Ok(ResolvedStoreSource {
        locked_key: prefetch.hash,
        flake_source,
    })
}
