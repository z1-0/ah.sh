use crate::provider::{Language, ProviderShowSelector, ProviderType};
use crate::session::SessionKey;
use clap::{Parser, Subcommand};

const BEFORE_HELP: &str = "

    █████   ██  ██
   ██   ██  ██  ██
   ███████  ██████
   ██   ██  ██  ██
   ██   ██  ██  ██ .sh";

const ABOUT: &str = "Magic development environments powered by Nix";

const AFTER_LONG_HELP: &str = "\x1b[1;4mAliases:\x1b[0m
  ah          ->  ah use
  ah restore  ->  ah session restore
  ah update   ->  ah session update

Use \x1b[1;3mah <COMMAND> --help\x1b[0m for more information about a command.
";

#[derive(Parser)]
#[command(version, about = ABOUT, before_help = BEFORE_HELP, after_help = AFTER_LONG_HELP, disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List and inspect providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
    },

    /// Restore a session by index or ID
    #[command(hide = true)]
    Restore {
        /// Session index (1, 2, ...) or ID (8 hex chars)
        key: SessionKey,
    },

    /// Manage development sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Update session dependencies
    #[command(hide = true)]
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
