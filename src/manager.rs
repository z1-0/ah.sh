use crate::error::Result;
use crate::executor::execute_nix_develop;
use crate::provider::ProviderType;
use crate::session::SessionKey;
use crate::session::SessionService;
use crate::warning::AppWarning;
use std::convert::Infallible;
use std::io::{self, IsTerminal, Write};

pub struct Manager;

const PROVIDER_TABLE_NAME_WIDTH: usize = 15;

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
        let result = SessionService::create_session(provider_type, languages)?;
        print_warnings(&result.warnings);
        execute_nix_develop(result.session_dir, true)
    }

    pub fn list_providers() -> Result<()> {
        let providers = [ProviderType::Devenv, ProviderType::DevTemplates];

        println!(
            "{:<5} {:<width$}",
            "Index",
            "Provider",
            width = PROVIDER_TABLE_NAME_WIDTH
        );

        for (i, provider_type) in providers.iter().enumerate() {
            let provider = (*provider_type).into_shell_provider();
            println!(
                "{:<5} {:<width$}",
                i + 1,
                provider.name(),
                width = PROVIDER_TABLE_NAME_WIDTH
            );
        }

        Ok(())
    }

    pub fn show_provider(provider_type: ProviderType) -> Result<()> {
        Self::write_provider_languages(provider_type, false)
    }

    pub fn show_all_providers() -> Result<()> {
        let providers = [ProviderType::Devenv, ProviderType::DevTemplates];

        for (i, provider) in providers.iter().enumerate() {
            if i > 0 {
                println!();
            }
            Self::write_provider_languages(*provider, true)?;
        }

        Ok(())
    }

    fn write_provider_languages(provider_type: ProviderType, include_header: bool) -> Result<()> {
        use std::io::{ErrorKind, Write};

        let provider = provider_type.into_shell_provider();
        let provider_name = provider.name();

        let mut languages = provider.get_supported_languages()?;
        languages.sort();

        let map_by_language = crate::provider::language_map_for_display(provider_name)?;

        let mut out = std::io::stdout().lock();

        if include_header && let Err(e) = writeln!(out, "Provider: {provider_name}") {
            if e.kind() == ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e.into());
        }

        for lang in languages {
            let mapped_inputs = map_by_language.get(&lang).cloned().unwrap_or_default();

            let line = if mapped_inputs.is_empty() {
                lang
            } else {
                // Show mapped inputs in parentheses after the mapped name.
                format!("{} ({})", lang, mapped_inputs.join(","))
            };

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

    // Yellow prefix, reset after the bracketed warning label.
    format!("\x1b[33mwarning[{}]\x1b[0m: {}", w.code, w.message)
}

fn sorted_warnings_for_print(warnings: &[AppWarning]) -> Vec<&AppWarning> {
    // Sort deterministically by (code, message), and keep a stable order for exact ties.
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
