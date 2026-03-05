use crate::error::Result;
use crate::manager::Manager;
use crate::providers::ProviderType;
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
        /// ID or index (1, 2, ...) of the session to restore, or 'list' to show sessions
        #[arg(default_value = "list")]
        args: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Session { args }) = &cli.command {
        if args == "list" {
            Manager::list_sessions()?;
        } else {
            Manager::restore_session(args)?;
        }
        return Ok(());
    }

    Manager::create_session(cli.provider, cli.language)?;

    Ok(())
}
