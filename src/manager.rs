use crate::cmd::{nix_develop_of_path, nix_develop_of_session, nix_flake_update_of_session};
use crate::provider::{
    Language, ProviderShowSelector, ProviderType, get_flake_contents, to_supported_languages,
};
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

pub fn init(provider: ProviderType, languages: Vec<Language>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let flake_path = current_dir.join("flake.nix");
    if flake_path.exists() {
        if is_terminal() {
            if !ask_confirmation("flake.nix already exists. Backup and overwrite? [y/N]: ") {
                print_info("Cancelled.");
                return Ok(());
            }
        } else {
            print_warning("flake.nix already exists. Auto-backing up to flake.nix.bak");
        }

        let backup_path = current_dir.join("flake.nix.bak");
        std::fs::copy(&flake_path, &backup_path)?;
        print_success(format!(
            "Backed up existing flake.nix to {}",
            backup_path.display()
        ));
    }

    let supported = to_supported_languages(provider, &languages)?;
    let flake_contents = get_flake_contents(provider)(&supported)?;

    std::fs::write(&flake_path, flake_contents)?;
    print_success(format!("Created {}", flake_path.display()));

    print_bold("Entering develop shell...");
    nix_develop_of_path(provider, current_dir)
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

pub fn restore_session(key: &SessionKey) -> Result<()> {
    let session = session::service::resolve_session_dir(key)?;
    nix_develop_of_session(session, true)
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
