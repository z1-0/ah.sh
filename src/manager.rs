use crate::cmd::{nix_develop_of_session, nix_flake_update_of_session};
use crate::provider::{Language, ProviderShowSelector, ProviderType};
use crate::session::SessionKey;
use crate::{output::*, session};
use anyhow::Result;

pub fn clear_sessions() -> Result<()> {
    if is_terminal() && !ask_confirmation("This will remove all sessions. Continue? [y/N]: ") {
        print_info("Cancelled.");
        return Ok(());
    }

    let removed = session::service::clear_sessions()?;
    print_success(format!("Cleared {} session(s).", removed));
    Ok(())
}

pub fn list_provider() -> Result<()> {
    let providers = ProviderShowSelector::All.as_provider_types();
    print_provider_list(providers)?;
    Ok(())
}

pub fn list_sessions() -> Result<()> {
    let sessions = session::service::list_sessions()?;
    if sessions.is_empty() {
        print_info("No sessions found.");
        return Ok(());
    }

    print_sessions_list(&sessions);
    Ok(())
}

pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
    let Some(result) = session::service::remove_sessions(keys)? else {
        print_info("No sessions found.");
        return Ok(());
    };

    if !result.removed_ids.is_empty() {
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
            let session = session::service::resolve_session_dir(k)?;
            nix_develop_of_session(session, true)
        }
        None => {
            // Show session history for current directory
            let cwd = crate::paths::get_cwd()?;
            if let Ok(sessions_with_ts) = session::service::find_by_path(&cwd) {
                if !sessions_with_ts.is_empty() {
                    let (sessions, timestamps): (Vec<_>, Vec<_>) =
                        sessions_with_ts.into_iter().unzip();
                    print_session_history(&sessions, &timestamps);

                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let choice = input.trim();
                        if let Ok(idx) = choice.parse::<usize>() {
                            if idx > 0 && idx <= sessions.len() {
                                let session = &sessions[idx - 1];
                                print_session_found(
                                    &session.id,
                                    &session.provider.to_string(),
                                    &session.languages,
                                );
                                print_bold("Restoring develop shell...");
                                return nix_develop_of_session(session.clone(), true);
                            }
                        }
                    }
                    println!();
                }
            }
            print_info("No session history found for current directory.");
            Ok(())
        }
    }
}

pub fn show_provider(provider: ProviderShowSelector) -> Result<()> {
    let providers = provider.as_provider_types();
    print_provider_show(providers)?;
    Ok(())
}

pub fn update_session(key: Option<&SessionKey>) -> Result<()> {
    let session = match key {
        Some(k) => session::service::resolve_session_dir(k)?,
        None => {
            let current_id = crate::paths::read_current_session()?.ok_or_else(|| {
                anyhow::anyhow!("No current session. Specify a session with 'ah update <index|id>'")
            })?;
            session::service::resolve_session_dir(&SessionKey::Id(current_id))?
        }
    };

    let session_dir = session.get_dir()?;
    let lock_path = session_dir.join("flake.lock");

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
        nix_develop_of_session(session, false)?
    } else {
        print_info("Dependencies are already up to date.");
    }

    Ok(())
}

pub fn use_languages(provider_type: ProviderType, languages: Vec<Language>) -> Result<()> {
    match session::service::find_session(provider_type, &languages)? {
        Some(session) => {
            print_session_found(
                &session.id,
                &session.provider.to_string(),
                &session.languages,
            );
            print_bold("Restoring develop shell...");
            nix_develop_of_session(session, true)
        }
        None => {
            let lang_strings: Vec<String> = languages.iter().map(|l| l.to_string()).collect();
            print_no_session(&provider_type.to_string(), &lang_strings);
            print_bold("Creating develop shell...");
            let session = session::service::create_session(provider_type, languages)?;
            nix_develop_of_session(session, false)
        }
    }
}
