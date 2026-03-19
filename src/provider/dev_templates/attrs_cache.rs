use crate::error::Result;
use crate::paths::get_cache_dir;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::warning::AppWarning;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const ATTRS_CACHE_DIR: &str = "dev-templates-attrs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CachedAttrsRecord {
    locked_key: String,
    attrs: ShellAttrs,
}

pub fn load_cached_attrs(
    lang: &str,
    locked_key: &str,
) -> Result<(Option<ShellAttrs>, Option<AppWarning>)> {
    let cache_dir = get_cache_dir()?.join(ATTRS_CACHE_DIR);
    load_cached_attrs_from_dir(&cache_dir, lang, locked_key)
}

pub fn save_cached_attrs(lang: &str, locked_key: &str, attrs: &ShellAttrs) -> Option<AppWarning> {
    let cache_dir = match get_cache_dir() {
        Ok(dir) => dir.join(ATTRS_CACHE_DIR),
        Err(err) => {
            return Some(
                AppWarning::new("dev_templates.attrs_cache_write_failed", err.to_string())
                    .with_context("language", lang.to_string()),
            );
        }
    };

    save_cached_attrs_to_dir(&cache_dir, lang, locked_key, attrs)
}

fn cache_file_path(cache_dir: &Path, lang: &str) -> PathBuf {
    cache_dir.join(format!("{lang}.json"))
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
    if let Err(err) = fs::create_dir_all(cache_dir) {
        return Some(
            AppWarning::new("dev_templates.attrs_cache_write_failed", err.to_string())
                .with_context("language", lang.to_string())
                .with_context("path", cache_dir.display().to_string()),
        );
    }

    let serialized = match serde_json::to_string(&CachedAttrsRecord {
        locked_key: locked_key.to_string(),
        attrs: attrs.clone(),
    }) {
        Ok(value) => value,
        Err(err) => {
            return Some(
                AppWarning::new("dev_templates.attrs_cache_write_failed", err.to_string())
                    .with_context("language", lang.to_string()),
            );
        }
    };

    let cache_file = cache_file_path(cache_dir, lang);
    match fs::write(&cache_file, serialized) {
        Ok(()) => None,
        Err(err) => Some(
            AppWarning::new("dev_templates.attrs_cache_write_failed", err.to_string())
                .with_context("language", lang.to_string())
                .with_context("path", cache_file.display().to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_cache_dir(test_name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("ah-attrs-cache-{test_name}-{nonce}"))
    }

    fn sample_attrs() -> ShellAttrs {
        ShellAttrs {
            env: vec![(
                "RUST_SRC_PATH".to_string(),
                "\"/nix/store/rust-src\"".to_string(),
            )],
            extra_attrs: vec![("venvDir".to_string(), "\".venv\"".to_string())],
        }
    }

    #[test]
    fn cache_hit_requires_same_lang_and_locked_key() {
        let cache_dir = test_cache_dir("hit");
        let attrs = sample_attrs();

        let save_warning = save_cached_attrs_to_dir(&cache_dir, "rust", "keyA", &attrs);
        assert!(save_warning.is_none());

        let (loaded, warning) =
            load_cached_attrs_from_dir(&cache_dir, "rust", "keyA").expect("load should not fail");

        assert!(warning.is_none());
        assert_eq!(loaded.expect("cache should hit"), attrs);
    }

    #[test]
    fn cache_miss_when_locked_key_changes() {
        let cache_dir = test_cache_dir("key-miss");
        let attrs = sample_attrs();

        let save_warning = save_cached_attrs_to_dir(&cache_dir, "rust", "keyA", &attrs);
        assert!(save_warning.is_none());

        let (loaded, warning) =
            load_cached_attrs_from_dir(&cache_dir, "rust", "keyB").expect("load should not fail");

        assert!(warning.is_none());
        assert!(loaded.is_none());
    }

    #[test]
    fn read_failed_cache_degrades_to_miss_with_warning() {
        let cache_dir = test_cache_dir("read-failed");
        let cache_file = cache_file_path(&cache_dir, "rust");
        std::fs::create_dir_all(&cache_file).expect("cache file path can be occupied by directory");

        let (loaded, warning) =
            load_cached_attrs_from_dir(&cache_dir, "rust", "keyA").expect("load should not fail");

        assert!(loaded.is_none());
        let warning = warning.expect("read failure should emit warning");
        assert_eq!(warning.code, "dev_templates.attrs_cache_read_failed");
        assert!(
            warning
                .context
                .contains(&("language".to_string(), "rust".to_string()))
        );
        assert!(
            warning
                .context
                .iter()
                .any(|(k, v)| k == "path" && v == &cache_file.display().to_string())
        );
    }

    #[test]
    fn corrupted_cache_degrades_to_miss_with_warning() {
        let cache_dir = test_cache_dir("corrupted");
        std::fs::create_dir_all(&cache_dir).expect("cache dir should be creatable");
        std::fs::write(cache_file_path(&cache_dir, "rust"), "{invalid json")
            .expect("invalid cache should be writable");

        let (loaded, warning) =
            load_cached_attrs_from_dir(&cache_dir, "rust", "keyA").expect("load should not fail");

        assert!(loaded.is_none());
        let warning = warning.expect("corrupted cache should emit warning");
        assert_eq!(warning.code, "dev_templates.attrs_cache_corrupted");
        assert!(
            warning
                .context
                .contains(&("language".to_string(), "rust".to_string()))
        );
        assert!(warning.context.iter().any(|(k, _)| k == "path"));
    }
}
