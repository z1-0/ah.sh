use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::provider::ProviderType;

/// User configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Provider type: "devenv" or "dev-templates"
    pub provider: ProviderType,

    /// Custom shell path, leave empty to use the $SHELL environment variable
    #[serde(default)]
    pub shell: Option<String>,
}

impl AppConfig {
    /// Default configuration values (used to create the default config file)
    pub fn default_values() -> Self {
        Self {
            provider: ProviderType::DevTemplates,
            shell: None,
        }
    }
}

/// Load user configuration
///
/// - If config file doesn't exist, auto-create default config
/// - If config file exists, load and validate
/// - Return clear error messages on config errors
pub fn load_config() -> Result<AppConfig> {
    let config_path =
        crate::path::config::get_config_file().context("Failed to determine config file path")?;

    // First use: copy the default config file
    if !config_path.exists() {
        create_default_config(&config_path).context("Failed to create default config")?;
    }

    // Load and validate configuration
    let config = ConfigBuilder::builder()
        .add_source(
            File::from(config_path.as_path())
                .format(FileFormat::Toml)
                .required(true),
        )
        .build()
        .context("Failed to build config loader")?
        .try_deserialize()
        .context(
            "Failed to parse config.toml. \
             Check syntax and field types. \
             See config.schema.json for reference",
        )?;

    Ok(config)
}

/// Copy default config from assets to user directory
fn create_default_config(dest_path: &std::path::Path) -> Result<()> {
    use std::fs;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    // Embed default config content
    let default_config = include_str!("assets/default_config.toml");
    fs::write(dest_path, default_config).context("Failed to write default config file")?;

    Ok(())
}
