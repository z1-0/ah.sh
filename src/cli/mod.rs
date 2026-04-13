mod implicit_use;
mod types;

use crate::cli::implicit_use::maybe_implicit_use_command;
use crate::manager;
use anyhow::{Context, Result};
use clap::CommandFactory;
use clap::Parser;

use types::{Cli, Commands, ProviderCommands, SessionCommands};

pub fn run() -> Result<()> {
    // Preload config: first run will auto-create ~/.config/ah/config.toml
    let _config = crate::config::load_config().context("Failed to load configuration")?;

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
        } => manager::use_languages(provider, languages),

        Commands::Provider { command } => match command {
            ProviderCommands::List => manager::list_provider(),
            ProviderCommands::Show { provider } => manager::show_provider(provider),
        },

        Commands::Session { command } => match command {
            SessionCommands::Clear => manager::clear_sessions(),
            SessionCommands::List => manager::list_sessions(),
            SessionCommands::Remove { keys } => manager::remove_sessions(&keys),
            SessionCommands::Restore { key } => manager::restore_session(key.as_ref()),
            SessionCommands::Update { session } => manager::update_session(session.as_ref()),
        },

        Commands::Restore { key } => manager::restore_session(key.as_ref()),
        Commands::Update { session } => manager::update_session(session.as_ref()),
    }
}
