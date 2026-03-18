use crate::error::Result;
use crate::warning::AppWarning;
use std::path::Path;

#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderType {
    Devenv,
    DevTemplates,
}

/// Target of `ah provider show`: select a provider, or choose all.
#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderKeyOrAll {
    Devenv,
    DevTemplates,
    All,
}

impl ProviderKeyOrAll {
    pub fn as_provider_type(&self) -> Option<ProviderType> {
        match self {
            ProviderKeyOrAll::Devenv => Some(ProviderType::Devenv),
            ProviderKeyOrAll::DevTemplates => Some(ProviderType::DevTemplates),
            ProviderKeyOrAll::All => None,
        }
    }
}

impl From<ProviderType> for ProviderKeyOrAll {
    fn from(value: ProviderType) -> Self {
        match value {
            ProviderType::Devenv => ProviderKeyOrAll::Devenv,
            ProviderType::DevTemplates => ProviderKeyOrAll::DevTemplates,
        }
    }
}

impl From<ProviderKeyOrAll> for Option<ProviderType> {
    fn from(value: ProviderKeyOrAll) -> Self {
        value.as_provider_type()
    }
}

pub struct EnsureFilesResult {
    pub warnings: Vec<AppWarning>,
}

pub trait ShellProvider {
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult>;
}

pub fn validate_languages(languages: &[String], supported: &[String]) -> Result<()> {
    let supported_set: std::collections::HashSet<_> = supported.iter().collect();
    let invalids: Vec<_> = languages
        .iter()
        .filter(|language| !supported_set.contains(language))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(())
    } else {
        Err(crate::error::AppError::UnsupportedLanguages(invalids))
    }
}
