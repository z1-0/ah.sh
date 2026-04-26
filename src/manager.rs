use std::io::stdin;

use anyhow::Result;
use strum::IntoEnumIterator;
use tracing::{debug, instrument};

use crate::cmd::{nix_develop_of_session, nix_flake_update_of_session};
use crate::config;
use crate::output;
use crate::path::cache::sessions::FLAKE_LOCK_FILE;
use crate::path::cache::{clear_current_session, read_current_session};
use crate::provider::{Language, ProviderType};
use crate::session;
use crate::session::SessionKey;

#[instrument(skip_all)]
pub fn clear_sessions() -> Result<()> {
    if output::is_interactive()
        && !output::ask_confirmation("This will remove all sessions. Continue? [y/N]: ")
    {
        output::print_bold("Cancelled.");
        return Ok(());
    }

    let removed = session::clear_sessions()?;
    if removed > 0 {
        clear_current_session();
    }
    output::print_success(format!("Cleared {} session(s).", removed));
    Ok(())
}

#[instrument(skip_all)]
pub fn list_provider() -> Result<()> {
    let providers = ProviderType::iter().collect::<Vec<_>>();
    output::print_provider_list(&providers);
    Ok(())
}

#[instrument(skip_all)]
pub fn list_sessions() -> Result<()> {
    let sessions = session::list_sessions()?;
    if sessions.is_empty() {
        output::print_bold("No sessions found.");
        return Ok(());
    }

    output::print_sessions_list(&sessions);
    Ok(())
}

#[instrument(skip_all)]
pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
    let Some(result) = session::remove_sessions(keys)? else {
        output::print_bold("No sessions found.");
        return Ok(());
    };

    if !result.removed_ids.is_empty() {
        if let Some(current_id) = read_current_session()?
            && result.removed_ids.contains(&current_id)
        {
            clear_current_session();
        }

        output::print_success(format!(
            "Removed {} session(s): {}",
            result.removed_ids.len(),
            result.removed_ids.join(", ")
        ));
    }
    if !result.missing_keys.is_empty() {
        output::print_error(format!("Not found: {}", result.missing_keys.join(", ")));
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn restore_session(key: Option<&SessionKey>) -> Result<()> {
    match key {
        Some(k) => {
            let session = session::find_session_by_key(k)?;
            nix_develop_of_session(session)
        }
        None => {
            if let Ok(sessions) = session::find_session_by_history()
                && !sessions.is_empty()
            {
                output::print_session_history(&sessions);

                let mut input = String::new();
                if stdin().read_line(&mut input).is_ok() {
                    let choice = input.trim();
                    if let Ok(idx) = choice.parse::<usize>()
                        && idx > 0
                        && idx <= sessions.len()
                    {
                        let session = &sessions[idx - 1];
                        debug!(session_id = %session.id, "User selected session from history");
                        output::print_bold("Restoring develop shell...");
                        return nix_develop_of_session(session.clone());
                    }
                }
                println!();
            } else {
                println!("No session history found for current directory.");
            }
            Ok(())
        }
    }
}

#[instrument(skip_all)]
pub fn show_provider(provider: ProviderType) -> Result<()> {
    output::print_provider_show(&[provider]);
    Ok(())
}

#[instrument(skip_all)]
pub fn update_session(key: Option<&SessionKey>) -> Result<()> {
    let session = match key {
        Some(k) => session::find_session_by_key(k)?,
        None => {
            let current_id = read_current_session()?.ok_or_else(|| {
                anyhow::anyhow!("No current session. Specify a session with 'ah update <index|id>'")
            })?;
            session::find_session_by_key(&SessionKey::Id(current_id))?
        }
    };

    let session_dir = session.get_dir();
    let lock_path = session_dir.join(FLAKE_LOCK_FILE);

    let mtime_before = lock_path.metadata().and_then(|m| m.modified()).ok();

    output::print_bold("Updating flake dependencies...");
    nix_flake_update_of_session(&session)?;
    session::touch_last_updated_at(&session)?;

    let mtime_after = lock_path.metadata().and_then(|m| m.modified()).ok();
    let was_updated = match (mtime_before, mtime_after) {
        (Some(before), Some(after)) => after > before,
        (None, Some(_)) => true,
        _ => false,
    };

    if was_updated {
        output::print_success("Dependencies updated.");
        output::print_bold("Entering develop shell...");
        nix_develop_of_session(session)?
    } else {
        output::print_bold("Dependencies are already up to date.");
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn use_languages(provider: Option<ProviderType>, languages: Vec<Language>) -> Result<()> {
    let provider = provider.unwrap_or(config::get().provider);
    match session::find_session(provider, &languages)? {
        Some(session) => {
            output::print_bold("Restoring develop shell...");
            nix_develop_of_session(session)
        }
        None => {
            output::print_bold("Creating develop shell...");
            let session = session::create_session(provider, languages)?;
            nix_develop_of_session(session)
        }
    }
}
