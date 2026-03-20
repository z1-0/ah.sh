use crate::error::Result;
use crate::executor::execute_nix_develop;
use crate::provider::{ProviderType, all_providers, provider_info};
use crate::session::SessionKey;
use crate::session::SessionService;
use crate::warning::AppWarning;
use std::convert::Infallible;
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

    pub fn restore_session(key: &SessionKey) -> Result<Infallible> {
        let session_dir = SessionService::resolve_session_dir(key)?;
        execute_nix_develop(session_dir, false)
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
            println!("Not found: {}", result.missing_keys.join(", "));
        }

        Ok(())
    }

    pub fn use_languages(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<Infallible> {
        // First try to find an existing session
        match SessionService::find_session(provider_type, &languages)? {
            Some(session) => {
                println!("Restoring existing session: {}", session.session_id);
                execute_nix_develop(session.session_dir, false)
            }
            None => {
                println!("Creating new session...");
                let result = SessionService::create_session(provider_type, languages)?;
                print_warnings(&result.warnings);
                execute_nix_develop(result.session.session_dir, true)
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

        for (i, provider) in all_providers().iter().enumerate() {
            println!("{}", format_provider_row(i + 1, provider.name()));
        }

        Ok(())
    }

    pub fn show_provider(provider_type: ProviderType) -> Result<()> {
        Self::write_provider_languages(provider_info(provider_type), false)
    }

    pub fn show_all_providers() -> Result<()> {
        for (i, provider) in all_providers().iter().enumerate() {
            if i > 0 {
                println!();
            }
            Self::write_provider_languages(provider, true)?;
        }

        Ok(())
    }

    fn write_provider_languages(
        provider: &crate::provider::ProviderInfo,
        include_header: bool,
    ) -> Result<()> {
        use std::io::{ErrorKind, Write};

        let provider_name = provider.name();

        let mut languages = provider.supported_languages()?;
        languages.sort();

        let map_by_language = provider.display_language_map()?;

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

fn format_warning_line(w: &AppWarning, color: bool) -> String {
    if !color {
        return format!("warning[{}]: {}", w.code, w.message);
    }

    format!("\x1b[33mwarning[{}]\x1b[0m: {}", w.code, w.message)
}

fn sorted_warnings_for_print(warnings: &[AppWarning]) -> Vec<&AppWarning> {
    let mut warnings: Vec<(usize, &AppWarning)> = warnings.iter().enumerate().collect();
    warnings.sort_by(|(ia, a), (ib, b)| (a.code, &a.message, ia).cmp(&(b.code, &b.message, ib)));

    warnings.into_iter().map(|(_, w)| w).collect()
}

fn print_warnings(warnings: &[AppWarning]) {
    let color = io::stderr().is_terminal();

    for w in sorted_warnings_for_print(warnings) {
        eprintln!("{}", format_warning_line(w, color));
    }
}
