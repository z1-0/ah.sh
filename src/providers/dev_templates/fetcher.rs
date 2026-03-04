use crate::error::{AhError, Result};
use crate::paths::{XdgDir, get_xdg_dir};
use std::fs;

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
    if cache_file.exists() {
        if let Ok(metadata) = fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 24 * 60 * 60 {
                        return Ok(fs::read_to_string(&cache_file)?);
                    }
                }
            }
        }
    }

    // Fetch from GitHub
    let url = format!(
        "https://raw.githubusercontent.com/the-nix-way/dev-templates/main/{}/flake.nix",
        lang
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| AhError::Provider(format!("Failed to fetch flake for {}: {}", lang, e)))?;

    if response.status() != 200 {
        return Err(AhError::Provider(format!(
            "Failed to fetch flake for {}: HTTP {}",
            lang,
            response.status()
        )));
    }

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| AhError::Provider(format!("Failed to read response body: {}", e)))?;

    // Save to cache
    fs::write(&cache_file, &body)?;

    Ok(body)
}
