use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::provider::ProviderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub provider: ProviderType,

    #[serde(default)]
    pub shell: Option<String>,
}

pub fn load_config() -> Result<AppConfig> {
    let config_path =
        crate::path::config::get_config_file().context("Failed to determine config file path")?;

    if !config_path.exists() {
        create_default_config(&config_path).context("Failed to create default config")?;
    }

    let config = ConfigBuilder::builder()
        .add_source(
            File::from(config_path.as_path())
                .format(FileFormat::Toml)
                .required(true),
        )
        .add_source(Environment::with_prefix("AH"))
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

fn create_default_config(dest_path: &std::path::Path) -> Result<()> {
    use std::fs;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let default_config = include_str!("assets/default_config.toml");
    fs::write(dest_path, default_config).context("Failed to write default config file")?;

    Ok(())
}
