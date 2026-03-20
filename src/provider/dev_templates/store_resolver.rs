use crate::error::{AppError, Result};
use serde_json::Value;
use std::fs;

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String>;
}

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

pub fn extract_locked_key(raw: &str) -> Result<String> {
    let parsed: Value = serde_json::from_str(raw)?;

    parsed
        .get("narHash")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("rev").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("lock key missing".to_string()))
}

pub fn resolve_store_source(lang: &str, runner: &dyn CommandRunner) -> Result<ResolvedStoreSource> {
    resolve_store_source_with_reader(lang, runner, &FsFlakeReader)
}

fn resolve_store_source_with_reader(
    lang: &str,
    runner: &dyn CommandRunner,
    reader: &dyn FlakeReader,
) -> Result<ResolvedStoreSource> {
    let lock_raw = query_lock_data(lang, runner)?;

    let (locked_key, store_path) = match extract_lock_and_store_path(&lock_raw) {
        Ok(parts) => parts,
        Err(_) => {
            prefetch(lang, runner)?;
            let retried_lock_raw = query_lock_data(lang, runner)?;
            extract_lock_and_store_path(&retried_lock_raw)?
        }
    };

    let flake_source = read_store_flake_with_reader(&store_path, lang, reader)?;

    Ok(ResolvedStoreSource {
        locked_key,
        flake_source,
    })
}

fn query_lock_data(lang: &str, runner: &dyn CommandRunner) -> Result<String> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    runner
        .run("nix", &["flake", "prefetch", "--json", flake_ref.as_str()])
        .map_err(|err| map_command_failure(lang, "query lock data", err))
}

fn prefetch(lang: &str, runner: &dyn CommandRunner) -> Result<()> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    runner
        .run("nix", &["flake", "prefetch", flake_ref.as_str()])
        .map_err(|err| map_command_failure(lang, "prefetch flake input", err))?;
    Ok(())
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

fn map_command_failure(lang: &str, action: &str, err: AppError) -> AppError {
    let summary = summarize_error(&err);
    AppError::Provider(format!(
        "failed to {action} for language `{lang}`: {summary}"
    ))
}

fn summarize_error(err: &AppError) -> String {
    match err {
        AppError::Provider(message) => message
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .unwrap_or_else(|| "unknown provider error".to_string()),
        _ => err.to_string(),
    }
}

fn extract_lock_and_store_path(raw: &str) -> Result<(String, String)> {
    let parsed: Value = serde_json::from_str(raw)?;

    let key = parsed
        .get("locked")
        .and_then(Value::as_object)
        .and_then(|locked| {
            locked
                .get("narHash")
                .and_then(Value::as_str)
                .or_else(|| locked.get("rev").and_then(Value::as_str))
        })
        .or_else(|| parsed.get("narHash").and_then(Value::as_str))
        .or_else(|| parsed.get("rev").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("lock key missing".to_string()))?;

    let path = parsed
        .get("path")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("storePath").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("store path missing".to_string()))?;

    Ok((key, path))
}
