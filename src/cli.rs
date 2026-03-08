use crate::error::Result;
use crate::manager::Manager;
use crate::providers::ProviderType;
use crate::sessions::SessionSelector;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,

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
        selector: SessionSelector,
    },
    /// Remove one or more sessions by index or id
    Remove {
        /// Session index(es) or id(s) (8 hex chars)
        #[arg(required = true, num_args = 1..)]
        targets: Vec<SessionSelector>,
    },
    /// Remove all sessions
    Clear,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Session { action }) = &cli.command {
        match action {
            None | Some(SessionCommands::List) => Manager::list_sessions()?,
            Some(SessionCommands::Restore { selector }) => Manager::restore_session(selector)?,
            Some(SessionCommands::Clear) => Manager::clear_sessions()?,
            Some(SessionCommands::Remove { targets }) => Manager::remove_sessions(targets)?,
        }
        return Ok(());
    }

    Manager::create_session(cli.provider, cli.language)?;

    Ok(())
}
