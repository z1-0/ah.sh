use std::io;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use crate::manager;

mod types;
use types::{Cli, Commands, ProviderCommands, SessionCommands};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => match cli.languages {
            Some(languages) => manager::use_languages(cli.provider, languages),
            None => Ok(Cli::command().print_long_help()?),
        },
        Some(command) => match command {
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
            Commands::Completion { shell } => {
                let mut cmd = Cli::command();
                let name = cmd.get_name().to_string();
                clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
                Ok(())
            }
        },
    }
}
