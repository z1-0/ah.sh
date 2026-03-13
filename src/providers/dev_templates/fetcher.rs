use crate::error::{AppError, Result};
use crate::paths::{XdgDir, get_xdg_dir};
use crate::warning::AppWarning;
use std::fs;
use std::path::Path;

const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

pub fn fetch_flake_source(lang: &str) -> Result<String> {
    // Check cache first
    let cache_dir = get_xdg_dir(XdgDir::Cache)?.join("dev-templates-source");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    let cache_file = cache_dir.join(format!("{}.nix", lang));

    // For simplicity, we just fetch it every time in development, but in a real
    // app we might want to cache it based on ETag or a TTL.
    // We'll cache it for 24 hours.
    if cache_file.exists()
        && let Ok(metadata) = fs::metadata(&cache_file)
        && let Ok(modified) = metadata.modified()
        && let Ok(elapsed) = modified.elapsed()
        && elapsed.as_secs() < CACHE_TTL_SECS
    {
        return Ok(fs::read_to_string(&cache_file)?);
    }

    // Fetch from GitHub
    let url = format!(
        "https://raw.githubusercontent.com/the-nix-way/dev-templates/main/{}/flake.nix",
        lang
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| AppError::Provider(format!("Failed to fetch flake for {}: {}", lang, e)))?;

    if response.status() != 200 {
        return Err(AppError::Provider(format!(
            "Failed to fetch flake for {}: HTTP {}",
            lang,
            response.status()
        )));
    }

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| AppError::Provider(format!("Failed to read response body: {}", e)))?;

    // Save to cache (best effort)
    let _ = write_cache_best_effort(&cache_file, &body);

    Ok(body)
}

#[cfg(test)]
pub(super) fn write_cache_best_effort_for_test(
    cache_file: &Path,
    body: &str,
) -> Option<AppWarning> {
    write_cache_best_effort(cache_file, body)
}

fn write_cache_best_effort(cache_file: &Path, body: &str) -> Option<AppWarning> {
    match fs::write(cache_file, body) {
        Ok(()) => None,
        Err(e) => Some(
            AppWarning::new("dev_templates.cache_write_failed", e.to_string())
                .with_context("path", cache_file.display().to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::write_cache_best_effort_for_test;
    use crate::warning::AppWarning;
    use std::path::Path;

    fn write_cache_best_effort(cache_file: &Path, body: &str) -> Option<AppWarning> {
        write_cache_best_effort_for_test(cache_file, body)
    }

    #[test]
    fn cache_write_best_effort_succeeds_when_path_is_writable() {
        let dir = tempfile::tempdir().unwrap();
        let cache_file = dir.path().join("cache.nix");

        let warning = write_cache_best_effort(&cache_file, "hello");

        assert!(warning.is_none());
        let content = std::fs::read_to_string(&cache_file).expect("cache file should be written");
        assert_eq!(content, "hello");
    }

    #[test]
    fn cache_write_best_effort_returns_warning_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let warning = write_cache_best_effort(dir.path(), "hello").expect("should warn");
        assert_eq!(warning.code, "dev_templates.cache_write_failed");
    }
}
