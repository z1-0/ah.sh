use crate::provider::{Language, ProviderShowSelector, ProviderType};
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
    /// 在当前目录生成 flake.nix 并进入开发环境
    Init {
        /// Languages to enable (e.g. rust go)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<Language>,
        /// Provider for flake generation
        #[arg(short, long, value_enum, default_value = "dev-templates")]
        provider: ProviderType,
    },

    /// Inspect available providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
    },

    /// Manage development sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Update session dependencies (nix flake update)
    Update {
        /// Session index (1, 2, ...) or id (8 hex chars). Uses current session if not specified.
        session: Option<SessionKey>,
    },

    /// Create a development session
    Use {
        /// Languages to enable (e.g. rust go)
        #[arg(required = true, num_args = 1..)]
        languages: Vec<Language>,

        /// Provider for session creation
        #[arg(short, long, value_enum, default_value = "dev-templates")]
        provider: ProviderType,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// Remove all sessions
    Clear,

    /// List sessions
    List,

    /// Remove one or more sessions by index or id
    Remove {
        /// Session index(es) or id(s) (8 hex chars)
        #[arg(required = true, num_args = 1..)]
        keys: Vec<SessionKey>,
    },

    /// Restore a session by index or id
    Restore {
        /// Session index (1, 2, ...) or id (8 hex chars)
        key: SessionKey,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// List all providers
    List,

    /// Show provider supported languages
    Show {
        /// Provider name (devenv/dev-templates) or "all"
        provider: ProviderShowSelector,
    },
}
