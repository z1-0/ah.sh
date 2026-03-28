mod implicit_use;
mod types;

use crate::cli::implicit_use::maybe_implicit_use_command;
use crate::manager::Manager;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;

use types::{Cli, Commands, ProviderCommands, SessionCommands};

pub fn run() -> Result<()> {
    let args = preprocess_args();
    let cli = Cli::try_parse_from(args)?;
    handle_command(cli.command)
}

/// Preprocesses command-line arguments to support the implicit use command.
fn preprocess_args() -> Vec<std::ffi::OsString> {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let mut cmd = Cli::command();
    cmd.build();
    maybe_implicit_use_command(args, &cmd)
}

/// Dispatches the parsed command to the appropriate manager logic.
fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Use {
            languages,
            provider,
        } => Manager::use_languages(provider, languages),

        Commands::Session { command } => match command {
            SessionCommands::List => Manager::list_sessions(),
            SessionCommands::Restore { key } => Manager::restore_session(&key),
            SessionCommands::Clear => Manager::clear_sessions(),
            SessionCommands::Remove { keys } => Manager::remove_sessions(&keys),
        },

        Commands::Provider { command } => match command {
            ProviderCommands::List => Manager::list_provider(),
            ProviderCommands::Show { provider } => Manager::show_provider(provider),
        },
    }
}
