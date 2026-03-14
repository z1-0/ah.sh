use crate::error::{AppError, Result};
use crate::manager::Manager;
use crate::providers::ProviderType;
use crate::session::SessionKey;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Provider for session creation (not used under `ah provider`)
    #[arg(short, long, value_enum, default_value = "dev-templates")]
    pub provider: ProviderType,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a development session
    Create {
        /// Languages to enable (e.g. rust go)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<String>,
    },

    /// Manage development sessions
    Session {
        #[command(subcommand)]
        action: Option<SessionCommands>,
    },

    /// Inspect available providers
    Provider {
        #[command(subcommand)]
        action: Option<ProviderCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List sessions
    List,
    /// Restore a session by index or id
    Restore {
        /// Session index (1, 2, ...) or id (8 hex chars)
        key: SessionKey,
    },
    /// Remove one or more sessions by index or id
    Remove {
        /// Session index(es) or id(s) (8 hex chars)
        #[arg(required = true, num_args = 1..)]
        keys: Vec<SessionKey>,
    },
    /// Remove all sessions
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// List all providers
    List,

    /// Show provider supported languages, Provider name (devenv/dev-templates) or "all"
    Show {
        /// Provider name (devenv/dev-templates) or "all"
        provider: ProviderShowTarget,
    },
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProviderShowTarget {
    Devenv,
    DevTemplates,
    All,
}

impl ProviderShowTarget {
    fn as_provider_type(&self) -> Option<ProviderType> {
        match self {
            ProviderShowTarget::Devenv => Some(ProviderType::Devenv),
            ProviderShowTarget::DevTemplates => Some(ProviderType::DevTemplates),
            ProviderShowTarget::All => None,
        }
    }
}

pub fn run() -> Result<()> {
    let cmd = Cli::command();
    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches)
        .map_err(|e: clap::Error| AppError::CliUsage(e.to_string()))?;

    if matches.subcommand_name() == Some("provider") {
        let provider_source = matches.value_source("provider");
        if matches!(
            provider_source,
            Some(clap::parser::ValueSource::CommandLine)
        ) {
            return Err(AppError::CliUsage(
                "`--provider/-p` is not supported under `ah provider`".to_string(),
            ));
        }
    }

    match &cli.command {
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()
                .map_err(|e| AppError::CliUsage(e.to_string()))?;
            println!();
            Ok(())
        }

        Some(Commands::Create { languages }) => {
            let never = Manager::create_session(cli.provider, languages.clone())?;
            match never {}
        }

        Some(Commands::Session { action }) => {
            match action {
                None | Some(SessionCommands::List) => Manager::list_sessions()?,
                Some(SessionCommands::Restore { key }) => {
                    let never = Manager::restore_session(&key)?;
                    match never {}
                }
                Some(SessionCommands::Clear) => Manager::clear_sessions()?,
                Some(SessionCommands::Remove { keys }) => Manager::remove_sessions(&keys)?,
            }
            Ok(())
        }

        Some(Commands::Provider { action }) => match action {
            None => {
                let mut cmd = Cli::command();
                cmd.find_subcommand_mut("provider")
                    .ok_or_else(|| AppError::Generic("provider subcommand not found".into()))?
                    .print_help()
                    .map_err(|e| AppError::CliUsage(e.to_string()))?;
                println!();
                Ok(())
            }
            Some(ProviderCommands::List) => Manager::list_providers(),
            Some(ProviderCommands::Show { provider: name }) => match name.as_provider_type() {
                None => Manager::show_all_providers(),
                Some(provider) => Manager::show_provider(provider),
            },
        },
    }
}
