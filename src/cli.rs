use crate::command::exec_nix_develop;
use crate::error::{AhError, Result};
use crate::providers::ShellProvider;
use crate::providers::dev_templates::DevTemplatesProvider;
use crate::providers::devenv::DevenvProvider;
use clap::{Parser, ValueEnum};
use std::collections::HashSet;

#[derive(ValueEnum, Clone, Debug)]
pub enum ProviderType {
    Devenv,
    DevTemplates,
}

impl ProviderType {
    pub fn into_shell_provider(self) -> Box<dyn ShellProvider> {
        match self {
            ProviderType::Devenv => Box::new(DevenvProvider::default()),
            ProviderType::DevTemplates => Box::new(DevTemplatesProvider::default()),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,

    #[arg(long, value_enum, default_value = "dev-templates")]
    pub provider: ProviderType,
}

pub fn languages() -> Result<()> {
    let cli = Cli::parse();
    let provider = cli.provider.into_shell_provider();

    // 1. Normalize and validate languages
    let normalized_langs = cli
        .language
        .iter()
        .map(|l| provider.normalize_language(l))
        .collect::<Vec<_>>();

    let supported_langs = provider.get_supported_languages()?;
    validate_languages(&normalized_langs, &supported_langs)?;

    // 2. Prepare environment resources
    let env_json = serde_json::to_string(&normalized_langs)?;
    let provider_path = provider.ensure_files()?;
    let path_str = provider_path
        .to_str()
        .ok_or_else(|| AhError::InvalidPath(provider_path.clone()))?;

    // 3. Execute
    exec_nix_develop(path_str, env_json);

    Ok(())
}

fn validate_languages(langs: &[String], supported: &[String]) -> Result<()> {
    let supported_set: HashSet<_> = supported.iter().collect();
    let invalids: Vec<_> = langs
        .iter()
        .filter(|l| !supported_set.contains(l))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(())
    } else {
        Err(AhError::UnsupportedLanguages(invalids))
    }
}
