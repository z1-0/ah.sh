use crate::cmd::nix_develop;
use crate::provider::{
    ProviderType, all_provider_types, provider_language_map_for_display, provider_name,
    supported_languages,
};
use crate::session::SessionKey;
use crate::session::SessionService;
use anyhow::Result;
use std::io::{self, IsTerminal, Write};

pub struct Manager;

const PROVIDER_TABLE_NAME_WIDTH: usize = 15;

fn format_provider_row(index: usize, provider_name: &str) -> String {
    format!(
        "{:<5} {:<width$}",
        index,
        provider_name,
        width = PROVIDER_TABLE_NAME_WIDTH
    )
}

fn format_provider_language_line(lang: String, mapped_inputs: Vec<String>) -> String {
    if mapped_inputs.is_empty() {
        lang
    } else {
        format!("{} ({})", lang, mapped_inputs.join(","))
    }
}

impl Manager {
    pub fn list_sessions() -> Result<()> {
        let sessions = SessionService::list_sessions()?;
        if sessions.is_empty() {
            println!("No sessions found.");
            return Ok(());
        }
        println!("{:<5} {:<10} {:<15} Languages", "Index", "ID", "Provider");
        for (i, s) in sessions.iter().enumerate() {
            println!(
                "{:<5} {:<10} {:<15} {}",
                i + 1,
                s.id,
                s.provider,
                s.languages.join(", ")
            );
        }
        Ok(())
    }

    pub fn restore_session(key: &SessionKey) -> Result<()> {
        let session_dir = SessionService::resolve_session_dir(key)?;
        nix_develop(session_dir, false)
    }

    pub fn clear_sessions() -> Result<()> {
        let should_confirm = io::stdin().is_terminal();
        if should_confirm {
            print!("This will remove all sessions. Continue? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let confirmed = matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes");
            if !confirmed {
                println!("Cancelled.");
                return Ok(());
            }
        }

        let removed = SessionService::clear_sessions()?;
        println!("Cleared {} session(s).", removed);
        Ok(())
    }

    pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
        let Some(result) = SessionService::remove_sessions(keys)? else {
            println!("No sessions found.");
            return Ok(());
        };

        if !result.removed_ids.is_empty() {
            println!(
                "Removed {} session(s): {}",
                result.removed_ids.len(),
                result.removed_ids.join(", ")
            );
        }
        if !result.missing_keys.is_empty() {
            tracing::warn!("Not found: {}", result.missing_keys.join(", "));
        }

        Ok(())
    }

    pub fn use_languages(provider_type: ProviderType, languages: Vec<String>) -> Result<()> {
        // First try to find an existing session
        match SessionService::find_session(provider_type, &languages)? {
            Some(session) => {
                tracing::info!("Restoring develop shell...");
                nix_develop(session.get_dir()?, true)
            }
            None => {
                tracing::info!("Creating develop shell...");
                let session = SessionService::create_session(provider_type, languages)?;
                nix_develop(session.get_dir()?, false)
            }
        }
    }

    pub fn list_providers() -> Result<()> {
        println!(
            "{:<5} {:<width$}",
            "Index",
            "Provider",
            width = PROVIDER_TABLE_NAME_WIDTH
        );

        for (i, provider_type) in all_provider_types().iter().enumerate() {
            println!(
                "{}",
                format_provider_row(i + 1, provider_name(*provider_type))
            );
        }

        Ok(())
    }

    pub fn show_provider(provider_type: ProviderType) -> Result<()> {
        Self::write_provider_languages(provider_type, false)
    }

    pub fn show_all_providers() -> Result<()> {
        for (i, provider_type) in all_provider_types().iter().enumerate() {
            if i > 0 {
                println!();
            }
            Self::write_provider_languages(*provider_type, true)?;
        }

        Ok(())
    }

    fn write_provider_languages(provider_type: ProviderType, include_header: bool) -> Result<()> {
        use std::io::{ErrorKind, Write};

        let provider_name = provider_name(provider_type);

        let mut languages = supported_languages(provider_type)?;
        languages.sort();

        let map_by_language = provider_language_map_for_display(provider_type)?;

        let mut out = std::io::stdout().lock();

        if include_header && let Err(e) = writeln!(out, "Provider: {provider_name}") {
            if e.kind() == ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e.into());
        }

        for lang in languages {
            let mapped_inputs = map_by_language.get(&lang).cloned().unwrap_or_default();
            let line = format_provider_language_line(lang, mapped_inputs);

            if let Err(e) = writeln!(out, "{line}") {
                if e.kind() == ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(e.into());
            }
        }

        Ok(())
    }
}
