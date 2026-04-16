mod implicit_use;
mod types;

use crate::cli::implicit_use::maybe_implicit_use_command;
use crate::manager;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;

use types::{Cli, Commands, ProviderCommands, SessionCommands};

pub fn run() -> Result<()> {
    let args = preprocess_args();
    let cli = Cli::try_parse_from(args)?;
    crate::cmd::check_nix_available()?;
    handle_command(cli.command)
}

fn preprocess_args() -> Vec<std::ffi::OsString> {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let mut cmd = Cli::command();
    cmd.build();
    maybe_implicit_use_command(args, &cmd)
}

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
