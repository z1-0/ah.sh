use crate::error::Result;
use crate::manager::Manager;
use crate::providers::{ProviderKeyOrAll, ProviderType};
use crate::session::SessionKey;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Provider for session creation (not used under `ah provider`)
    #[arg(short, long, value_enum, default_value = "dev-templates")]
    pub provider: ProviderType,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a development session
    Use {
        /// Languages to enable (e.g. rust go)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<String>,
    },

    /// Manage development sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Inspect available providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
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

    /// Show provider supported languages
    Show {
        /// Provider name (devenv/dev-templates) or "all"
        provider: ProviderKeyOrAll,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Use { languages } => {
            let never = Manager::use_languages(cli.provider, languages.clone())?;
            match never {}
        }

        Commands::Session { command } => match command {
            SessionCommands::List => Manager::list_sessions(),
            SessionCommands::Restore { key } => {
                let never = Manager::restore_session(key)?;
                match never {}
            }
            SessionCommands::Clear => Manager::clear_sessions(),
            SessionCommands::Remove { keys } => Manager::remove_sessions(keys),
        },

        Commands::Provider { command } => match command {
            ProviderCommands::List => Manager::list_providers(),
            ProviderCommands::Show { provider } => match provider.as_provider_type() {
                None => Manager::show_all_providers(),
                Some(provider) => Manager::show_provider(provider),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_use_with_single_language() {
        let cli = Cli::try_parse_from(["ah", "use", "rust"]).unwrap();
        match cli.command {
            Commands::Use { languages } => assert_eq!(languages, vec!["rust"]),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_lang_is_rejected() {
        let err = Cli::try_parse_from(["ah", "lang", "rust"]).err().unwrap();
        assert!(err.to_string().contains("lang"));
    }

    #[test]
    fn parse_use_without_languages_is_rejected() {
        let err = Cli::try_parse_from(["ah", "use"]).err().unwrap();
        assert!(err.to_string().contains("use"));
    }
}
