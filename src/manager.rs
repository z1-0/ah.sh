use crate::app::SessionApp;
use crate::error::Result;
use crate::executor::execute_nix_develop;
use crate::providers::ProviderType;
use crate::session::SessionKey;
use crate::warning::AppWarning;
use std::convert::Infallible;
use std::io::{self, IsTerminal, Write};

pub struct Manager;

const PROVIDER_TABLE_NAME_WIDTH: usize = 15;

impl Manager {
    pub fn list_sessions() -> Result<()> {
        let sessions = SessionApp::list_sessions()?;
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
        let session_dir = SessionApp::prepare_restore_session(key)?;
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

        let removed = SessionApp::clear_sessions()?;
        println!("Cleared {} session(s).", removed);
        Ok(())
    }

    pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
        let Some(result) = SessionApp::remove_sessions(keys)? else {
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

    pub fn create_session(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<Infallible> {
        let result = SessionApp::prepare_create_session(provider_type, languages)?;
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

        let aliases_by_canonical =
            crate::providers::language_aliases_by_canonical_for_provider(provider_name)?;

        let mut out = std::io::stdout().lock();

        if include_header {
            if let Err(e) = writeln!(out, "Provider: {provider_name}") {
                if e.kind() == ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(e.into());
            }
        }

        for lang in languages {
            let aliases = aliases_by_canonical.get(&lang).cloned().unwrap_or_default();

            let line = if aliases.is_empty() {
                lang
            } else {
                // Show aliases in parentheses after the canonical name.
                // Same-name aliases are ignored by language_aliases_by_canonical_for_provider.
                format!("{} ({})", lang, aliases.join(","))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_warning_line_without_color_matches_legacy_format() {
        let w = AppWarning::new("W001", "hello");
        assert_eq!(format_warning_line(&w, false), "warning[W001]: hello");
    }

    #[test]
    fn format_warning_line_with_color_includes_ansi_and_original_text() {
        let w = AppWarning::new("W001", "hello");
        let s = format_warning_line(&w, true);

        assert!(s.contains("\x1b[33m"), "expected yellow ANSI code");
        assert!(s.contains("\x1b[0m"), "expected ANSI reset code");
        assert!(s.contains("warning[W001]"));
        assert!(s.ends_with(": hello"));
    }

    #[test]
    fn sorted_warnings_for_print_returns_borrowed_refs_in_sorted_order() {
        let warnings = vec![
            AppWarning::new("W002", "other"),
            AppWarning::new("W001", "duplicate"),
            AppWarning::new("W001", "aaa"),
            AppWarning::new("W001", "duplicate"),
        ];

        let sorted = sorted_warnings_for_print(&warnings);
        let ptrs: Vec<*const AppWarning> = sorted.iter().map(|w| *w as *const AppWarning).collect();

        // Sorting key is (code, message) with stable ordering for exact ties.
        let expected = vec![
            &warnings[2] as *const AppWarning, // (W001, aaa)
            &warnings[1] as *const AppWarning, // (W001, duplicate) - first
            &warnings[3] as *const AppWarning, // (W001, duplicate) - second
            &warnings[0] as *const AppWarning, // (W002, other)
        ];

        assert_eq!(ptrs, expected);
    }

    #[test]
    fn print_warnings_sorts_stably_when_tied() {
        let warnings = vec![
            AppWarning::new("W002", "other").with_context("id", "other"),
            AppWarning::new("W001", "duplicate").with_context("id", "first"),
            AppWarning::new("W001", "aaa").with_context("id", "aaa"),
            AppWarning::new("W001", "duplicate").with_context("id", "second"),
        ];

        let sorted = sorted_warnings_for_print(&warnings);
        let tied_ids: Vec<&str> = sorted
            .iter()
            .filter(|w| w.code == "W001" && w.message == "duplicate")
            .map(|w| {
                w.context
                    .iter()
                    .find(|(k, _)| k == "id")
                    .unwrap()
                    .1
                    .as_str()
            })
            .collect();

        assert_eq!(tied_ids, vec!["first", "second"]);
    }
}
