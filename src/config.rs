use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, File, FileFormat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::paths::config::get_config_path;
use crate::provider::ProviderType;

/// 用户配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppConfig {
    /// 提供商类型: "devenv" 或 "dev-templates"
    pub provider: ProviderType,

    /// 自定义 shell 路径，留空则使用 $SHELL 环境变量
    #[serde(default)]
    pub shell: Option<String>,
}

impl AppConfig {
    /// 默认配置值（用于创建默认配置文件）
    pub fn default_values() -> Self {
        Self {
            provider: ProviderType::DevTemplates,
            shell: None,
        }
    }
}

/// 加载用户配置
///
/// - 如果配置文件不存在，自动创建默认配置
/// - 如果配置文件存在，加载并验证
/// - 配置错误时返回清晰的错误信息
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path().context("Failed to determine config file path")?;

    // 首次使用：复制默认配置文件
    if !config_path.exists() {
        create_default_config(&config_path).context("Failed to create default config")?;
    }

    // 加载并验证配置
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

/// 从 assets 复制默认配置到用户目录
fn create_default_config(dest_path: &std::path::Path) -> Result<()> {
    use std::fs;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    // 嵌入默认配置内容
    let default_config = include_str!("assets/default_config.toml");
    fs::write(dest_path, default_config).context("Failed to write default config file")?;

    Ok(())
}
