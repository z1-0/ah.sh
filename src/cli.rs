use crate::error::Result;
use crate::manager::Manager;
use crate::providers::ProviderType;
use crate::session::SessionKey;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub languages: Vec<String>,

    #[arg(short, long, value_enum, default_value = "dev-templates")]
    pub provider: ProviderType,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage development sessions
    Session {
        #[command(subcommand)]
        action: Option<SessionCommands>,
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

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Session { action }) = &cli.command {
        match action {
            None | Some(SessionCommands::List) => Manager::list_sessions()?,
            Some(SessionCommands::Restore { key }) => Manager::restore_session(key)?,
            Some(SessionCommands::Clear) => Manager::clear_sessions()?,
            Some(SessionCommands::Remove { keys }) => Manager::remove_sessions(keys)?,
        }
        return Ok(());
    }

    Manager::create_session(cli.provider, cli.languages)?;

    Ok(())
}
