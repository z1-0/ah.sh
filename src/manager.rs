use crate::cmd::{nix_develop_of_session, nix_flake_update_of_session};
use crate::path::cache::sessions::FLAKE_LOCK_FILE;
use crate::path::cache::{clear_current_session, read_current_session};
use crate::provider::{Language, ProviderType};
use crate::session::SessionKey;
use crate::{output::*, session};
use strum::IntoEnumIterator;

use anyhow::Result;

pub fn clear_sessions() -> Result<()> {
    if is_terminal() && !ask_confirmation("This will remove all sessions. Continue? [y/N]: ") {
        print_info("Cancelled.");
        return Ok(());
    }

    let removed = session::clear_sessions()?;
    if removed > 0 {
        clear_current_session()?;
    }
    print_success(format!("Cleared {} session(s).", removed));
    Ok(())
}

pub fn list_provider() -> Result<()> {
    let providers = ProviderType::iter().collect::<Vec<_>>();
    print_provider_list(&providers)?;
    Ok(())
}

pub fn list_sessions() -> Result<()> {
    let sessions = session::list_sessions()?;
    if sessions.is_empty() {
        print_info("No sessions found.");
        return Ok(());
    }

    print_sessions_list(&sessions);
    Ok(())
}

pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
    let Some(result) = session::remove_sessions(keys)? else {
        print_info("No sessions found.");
        return Ok(());
    };

    if !result.removed_ids.is_empty() {
        if let Some(current_id) = read_current_session()?
            && result.removed_ids.contains(&current_id)
        {
            clear_current_session()?;
        }

        print_success(format!(
            "Removed {} session(s): {}",
            result.removed_ids.len(),
            result.removed_ids.join(", ")
        ));
    }
    if !result.missing_keys.is_empty() {
        print_error(format!("Not found: {}", result.missing_keys.join(", ")));
    }

    Ok(())
}

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
                print_session_history(&sessions);

                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    let choice = input.trim();
                    if let Ok(idx) = choice.parse::<usize>()
                        && idx > 0
                        && idx <= sessions.len()
                    {
                        let session = &sessions[idx - 1];
                        print_session_found(
                            &session.id,
                            &session.provider.to_string(),
                            &session.languages,
                        );
                        print_bold("Restoring develop shell...");
                        return nix_develop_of_session(session.clone());
                    }
                }
                println!();
            }
            print_info("No session history found for current directory.");
            Ok(())
        }
    }
}

pub fn show_provider(provider: ProviderType) -> Result<()> {
    print_provider_show(&[provider])?;
    Ok(())
}

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

    print_bold("Updating flake dependencies...");
    nix_flake_update_of_session(&session)?;

    let mtime_after = lock_path.metadata().and_then(|m| m.modified()).ok();
    let was_updated = match (mtime_before, mtime_after) {
        (Some(before), Some(after)) => after > before,
        (None, Some(_)) => true,
        _ => false,
    };

    if was_updated {
        print_success("Dependencies updated.");
        print_bold("Entering develop shell...");
        nix_develop_of_session(session)?
    } else {
        print_info("Dependencies are already up to date.");
    }

    Ok(())
}

pub fn use_languages(provider_type: ProviderType, languages: Vec<Language>) -> Result<()> {
    match session::find_session(provider_type, &languages)? {
        Some(session) => {
            print_session_found(
                &session.id,
                &session.provider.to_string(),
                &session.languages,
            );
            print_bold("Restoring develop shell...");
            nix_develop_of_session(session)
        }
        None => {
            print_no_session(&provider_type.to_string(), &languages);
            print_bold("Creating develop shell...");
            let session = session::create_session(provider_type, languages)?;
            nix_develop_of_session(session)
        }
    }
}
