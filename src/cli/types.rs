use crate::provider::{ProviderKeyOrAll, ProviderType};
use crate::session::SessionKey;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a development session
    Use {
        /// Languages to enable (e.g. rust go)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<String>,

        /// Provider for session creation
        #[arg(short, long, value_enum, default_value = "dev-templates")]
        provider: ProviderType,
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
