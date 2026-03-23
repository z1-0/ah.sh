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
    let cli = Cli::parse_from(args);
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
        } => {
            let never = Manager::use_languages(provider, languages)?;
            match never {}
        }

        Commands::Session { command } => match command {
            SessionCommands::List => Manager::list_sessions(),
            SessionCommands::Restore { key } => {
                let never = Manager::restore_session(&key)?;
                match never {}
            }
            SessionCommands::Clear => Manager::clear_sessions(),
            SessionCommands::Remove { keys } => Manager::remove_sessions(&keys),
        },

        Commands::Provider { command } => match command {
            ProviderCommands::List => Manager::list_providers(),
            ProviderCommands::Show { provider } => match provider.as_provider_type() {
                None => Manager::show_all_providers(),
                Some(p) => Manager::show_provider(p),
            },
        },
    }
}
