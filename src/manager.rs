use crate::app::SessionApp;
use crate::error::Result;
use crate::executor::execute_nix_develop;
use crate::providers::ProviderType;
use crate::session::SessionKey;
use crate::warning::AppWarning;
use std::convert::Infallible;
use std::io::{self, IsTerminal, Write};

pub struct Manager;

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
}

fn format_warning_line(w: &AppWarning, color: bool) -> String {
    if !color {
        return format!("warning[{}]: {}", w.code, w.message);
    }

    // Yellow prefix, reset after the bracketed warning label.
    format!("\x1b[33mwarning[{}]\x1b[0m: {}", w.code, w.message)
}

fn print_warnings(warnings: &[AppWarning]) {
    let color = io::stderr().is_terminal();

    // Sort deterministically by (code, message), and keep a stable order for exact ties.
    let mut warnings: Vec<(usize, AppWarning)> = warnings.iter().cloned().enumerate().collect();
    warnings.sort_by(|(ia, a), (ib, b)| {
        (a.code, &a.message, ia).cmp(&(b.code, &b.message, ib))
    });

    for (_, w) in warnings {
        eprintln!("{}", format_warning_line(&w, color));
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
}
