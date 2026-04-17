use std::sync::OnceLock;

use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File, FileFormat};

use crate::provider::ProviderType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AppConfig {
    pub provider: ProviderType,
    pub shell: Option<String>,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get() -> &'static AppConfig {
    CONFIG.get().unwrap()
}

pub fn load_config() -> Result<()> {
    let config_path = crate::path::config::get_config_file();

    if !config_path.exists() {
        create_default_config(&config_path)?
    }

    let config_data = ConfigBuilder::builder()
        .add_source(File::from(config_path.as_path()).format(FileFormat::Toml).required(true))
        .add_source(Environment::with_prefix(crate::APP_NAME))
        .build()
        .map_err(|e| match e {
            config::ConfigError::FileParse { .. } => {
                anyhow::Error::from(e).context("Failed to parse config.toml. Please check for TOML syntax errors.")
            }
            _ => {
                anyhow::Error::from(e).context("Failed to build configuration. Please check your environment variables.")
            }
        })?
        .try_deserialize::<AppConfig>()
        .context(
            "Configuration data is invalid. \
             Ensure all fields match the required types and structure. \
             See https://github.com/z1-0/ah.sh/blob/main/src/assets/config.schema.json for reference",
        )?;

    CONFIG
        .set(config_data)
        .map_err(|_| anyhow::anyhow!("Config already initialized"))?;

    Ok(())
}

fn create_default_config(dest_path: &std::path::Path) -> Result<()> {
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let default_config = include_str!("assets/default_config.toml");
    std::fs::write(dest_path, default_config).context("Failed to write default config file")?;

    Ok(())
}

#[test]
fn ensure_schema_is_up_to_date() {
    use std::path::PathBuf;

    let schema = schemars::schema_for!(AppConfig);
    let current_schema = serde_json::to_string_pretty(&schema).unwrap();

    let mut schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    schema_path.push("src");
    schema_path.push("assets");
    schema_path.push("config.schema.json");
    let existing_schema = std::fs::read_to_string(&schema_path).unwrap_or_default();

    if current_schema != existing_schema {
        std::fs::write(&schema_path, current_schema).unwrap();
        panic!("Schema was out of date and has been updated. Please commit the changes.");
    }
}
