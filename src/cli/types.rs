use crate::provider::{Language, ProviderShowSelector, ProviderType};
use crate::session::SessionKey;
use clap::{Parser, Subcommand};

const HELP_ASCII_ART: &str = r#"
    █████   ██  ██
   ██   ██  ██  ██
   ███████  ██████
   ██   ██  ██  ██
   ██   ██  ██  ██ .sh"#;

/// Magic shell environments powered by Nix
#[derive(Parser)]
#[command(version, about, before_help = HELP_ASCII_ART)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate flake.nix and enter dev environment
    Init {
        /// Languages to enable (e.g., rust go nodejs)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<Language>,
        /// Which provider to use
        #[arg(short, long, value_enum, default_value = "dev-templates")]
        provider: ProviderType,
    },

    /// List and inspect providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
    },

    /// Manage development sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Update session dependencies
    Update {
        /// Session index (1, 2, ...) or ID (8 hex chars). Uses current session if not specified
        session: Option<SessionKey>,
    },

    /// Create and enter a development environment
    Use {
        /// Languages to enable (e.g., rust go nodejs)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<Language>,

        /// Which provider to use
        #[arg(short, long, value_enum, default_value = "dev-templates")]
        provider: ProviderType,
    },

    /// Restore a session by index or ID
    Restore {
        /// Session index (1, 2, ...) or ID (8 hex chars)
        key: SessionKey,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// Delete all sessions
    Clear,

    /// List all sessions
    List,

    /// Delete one or more sessions by index or ID
    Remove {
        /// Session index(es) or ID(s) (8 hex chars)
        #[arg(required = true, num_args = 1..)]
        keys: Vec<SessionKey>,
    },

    /// Restore a session by index or ID
    Restore {
        /// Session index (1, 2, ...) or ID (8 hex chars)
        key: SessionKey,
    },

    /// Update session dependencies
    Update {
        /// Session index (1, 2, ...) or ID (8 hex chars). Uses current session if not specified
        session: Option<SessionKey>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// List all available providers
    List,

    /// Show supported languages for a provider
    Show {
        /// Provider name (devenv, dev-templates) or "all"
        provider: ProviderShowSelector,
    },
}
