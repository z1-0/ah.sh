use crate::error::Result;
use crate::paths::get_attrs_cache_dir;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::warning::AppWarning;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CachedAttrsRecord {
    locked_key: String,
    attrs: ShellAttrs,
}

pub fn load_cached_attrs(
    lang: &str,
    locked_key: &str,
) -> Result<(Option<ShellAttrs>, Option<AppWarning>)> {
    let cache_dir = get_attrs_cache_dir()?;
    load_cached_attrs_from_dir(&cache_dir, lang, locked_key)
}

pub fn save_cached_attrs(lang: &str, locked_key: &str, attrs: &ShellAttrs) -> Option<AppWarning> {
    let cache_dir = match get_attrs_cache_dir() {
        Ok(dir) => dir,
        Err(err) => return Some(write_failed_warning(lang, err)),
    };

    save_cached_attrs_to_dir(&cache_dir, lang, locked_key, attrs)
}

fn cache_file_path(cache_dir: &Path, lang: &str) -> PathBuf {
    cache_dir.join(format!("{lang}.json"))
}

fn write_failed_warning(lang: &str, err: impl ToString) -> AppWarning {
    AppWarning::new("dev_templates.attrs_cache_write_failed", err.to_string())
        .with_context("language", lang.to_string())
}

fn load_cached_attrs_from_dir(
    cache_dir: &Path,
    lang: &str,
    locked_key: &str,
) -> Result<(Option<ShellAttrs>, Option<AppWarning>)> {
    let cache_file = cache_file_path(cache_dir, lang);
    if !cache_file.exists() {
        return Ok((None, None));
    }

    let raw = match fs::read_to_string(&cache_file) {
        Ok(value) => value,
        Err(err) => {
            let warning = AppWarning::new("dev_templates.attrs_cache_read_failed", err.to_string())
                .with_context("language", lang.to_string())
                .with_context("path", cache_file.display().to_string());
            return Ok((None, Some(warning)));
        }
    };
    let parsed: CachedAttrsRecord = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => {
            let warning = AppWarning::new("dev_templates.attrs_cache_corrupted", err.to_string())
                .with_context("language", lang.to_string())
                .with_context("path", cache_file.display().to_string());
            return Ok((None, Some(warning)));
        }
    };

    if parsed.locked_key == locked_key {
        Ok((Some(parsed.attrs), None))
    } else {
        Ok((None, None))
    }
}

fn save_cached_attrs_to_dir(
    cache_dir: &Path,
    lang: &str,
    locked_key: &str,
    attrs: &ShellAttrs,
) -> Option<AppWarning> {
    let serialized = match serde_json::to_string(&CachedAttrsRecord {
        locked_key: locked_key.to_string(),
        attrs: attrs.clone(),
    }) {
        Ok(value) => value,
        Err(err) => return Some(write_failed_warning(lang, err)),
    };

    let cache_file = cache_file_path(cache_dir, lang);
    match fs::write(&cache_file, serialized) {
        Ok(()) => None,
        Err(err) => Some(
            write_failed_warning(lang, err).with_context("path", cache_file.display().to_string()),
        ),
    }
}
