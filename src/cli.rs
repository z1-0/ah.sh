use std::collections::HashSet;

use clap::{Parser, ValueEnum};

use crate::{
    command::exec_nix_develop,
    providers::{EnvironmentProvider, dev_templates::DevTemplatesProvider, devenv::DevenvProvider},
};

#[derive(ValueEnum, Clone)]
pub enum Provider {
    Devenv,
    DevTemplates,
}

impl Provider {
    pub fn to_provider_trait(&self) -> Box<dyn EnvironmentProvider> {
        match self {
            Provider::Devenv => Box::new(DevenvProvider),
            Provider::DevTemplates => Box::new(DevTemplatesProvider),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,

    #[arg(long, value_enum, default_value = "devenv")]
    pub provider: Provider,
}

pub fn languages() -> Result<(), String> {
    let cli = Cli::parse();
    let provider = cli.provider.to_provider_trait();

    let provider_dir = provider.get_dir();
    let provider_dir_str = provider_dir
        .to_str()
        .ok_or_else(|| "Invalid provider path".to_owned())?;

    let normalized_langs: Vec<String> = cli
        .language
        .iter()
        .map(|l| provider.normalize_language(l))
        .collect();

    let supported_langs = provider.get_supported_languages();
    let ensures = ensure_languages(normalized_langs, supported_langs)?;

    let env_ahsh_languages =
        serde_json::to_string(&ensures).expect("Failed to serialize languages");

    exec_nix_develop(provider_dir_str, env_ahsh_languages);
    Ok(())
}

fn ensure_languages(
    langs: Vec<String>,
    supported_langs: Vec<String>,
) -> Result<Vec<String>, String> {
    let supported: HashSet<String> = supported_langs.into_iter().collect();

    let invalids: Vec<String> = langs
        .iter()
        .filter(|&l| !supported.contains(l))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(langs)
    } else {
        Err(format!("Languages {:?} are not supported", invalids))
    }
}
