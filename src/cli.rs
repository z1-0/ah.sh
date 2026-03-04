use crate::command::{exec_nix_develop_with_provider, exec_nix_develop_with_session};
use crate::error::{AhError, Result};
use crate::providers::ShellProvider;
use crate::providers::dev_templates::DevTemplatesProvider;
use crate::providers::devenv::DevenvProvider;
use crate::sessions::{self, Session};
use clap::{Parser, Subcommand, ValueEnum};
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

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage development sessions
    Session {
        /// ID or index (1, 2, ...) of the session to restore, or 'list' to show sessions
        #[arg(default_value = "list")]
        args: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Session { args }) = &cli.command {
        if args == "list" {
            let list = sessions::list_sessions()?;
            if list.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }
            println!(
                "{:<5} {:<10} {:<15} {}",
                "ID", "Hash", "Provider", "Languages"
            );
            for (i, s) in list.iter().enumerate() {
                println!(
                    "{:<5} {:<10} {:<15} {}",
                    i + 1,
                    s.id,
                    s.provider,
                    s.languages.join(", ")
                );
            }
            return Ok(());
        } else {
            // Restore session
            let session = sessions::find_session(args)?;
            let profile_path = session.get_profile_path()?;

            exec_nix_develop_with_session(profile_path);
            return Ok(());
        }
    }

    let provider = cli.provider.into_shell_provider();

    // 1. Normalize and validate languages
    let normalized_langs = cli
        .language
        .iter()
        .map(|l| provider.normalize_language(l))
        .collect::<Vec<_>>();

    if normalized_langs.is_empty() {
        return Err(AhError::Generic(
            "No languages specified. Use 'ah <langs>' or 'ah session list'".to_string(),
        ));
    }

    let supported_langs = provider.get_supported_languages()?;
    validate_languages(&normalized_langs, &supported_langs)?;

    // 2. Prepare environment resources
    let env_json = serde_json::to_string(&normalized_langs)?;
    let provider_path = provider.ensure_files(&normalized_langs)?;
    let path_str = provider_path
        .to_str()
        .ok_or_else(|| AhError::InvalidPath(provider_path.clone()))?;

    // 3. Session Management
    let flake_path = provider_path.join("flake.nix");
    let flake_content = std::fs::read_to_string(flake_path)?;
    let session_id = sessions::generate_id(provider.name(), &normalized_langs, &flake_content);
    let session = Session::new(session_id, normalized_langs, provider.name().to_string());
    sessions::save_session(&session)?;

    let profile_path = session.get_profile_path()?;

    // 4. Execute
    exec_nix_develop_with_provider(path_str, env_json, profile_path);

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
