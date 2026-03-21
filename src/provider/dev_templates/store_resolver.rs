use crate::cmd;
use crate::error::{AppError, Result};
use std::fs;

trait FlakeReader {
    fn read_to_string(&self, path: &str) -> std::io::Result<String>;
}

struct FsFlakeReader;

impl FlakeReader for FsFlakeReader {
    fn read_to_string(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}

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
    resolve_store_source_with_reader(lang, &FsFlakeReader)
}

fn resolve_store_source_with_reader(
    lang: &str,
    reader: &dyn FlakeReader,
) -> Result<ResolvedStoreSource> {
    let lock_raw = cmd::nix_flake_prefetch(lang)?;
    let prefetch = parse_prefetch_response(&lock_raw)?;

    let flake_source = read_store_flake_with_reader(&prefetch.store_path, lang, reader)?;

    Ok(ResolvedStoreSource {
        locked_key: prefetch.hash,
        flake_source,
    })
}

fn parse_prefetch_response(raw: &str) -> Result<PrefetchResponse> {
    serde_json::from_str(raw).map_err(Into::into)
}

fn read_store_flake_with_reader(
    store_path: &str,
    lang: &str,
    reader: &dyn FlakeReader,
) -> Result<String> {
    let flake_path = format!("{store_path}/{lang}/flake.nix");
    reader
        .read_to_string(&flake_path)
        .map_err(|err| AppError::Provider(format!("failed to read {flake_path}: {err}")))
}
