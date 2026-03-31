use crate::cmd::exec;
use crate::output::*;
use crate::provider::{
    Language, ProviderShowSelector, ProviderType, get_flake_contents, to_supported_languages,
};
use crate::session::SessionService;
use crate::session::{Session, SessionKey};
use anyhow::Result;

fn nix_develop_new_session(session: Session) -> Result<()> {
    crate::cmd::nix_develop(session, false)
}

fn nix_develop_existing_session(session: Session) -> Result<()> {
    crate::cmd::nix_develop(session, true)
}

pub struct Manager;

impl Manager {
    pub fn clear_sessions() -> Result<()> {
        if is_terminal() && !ask_confirmation("This will remove all sessions. Continue? [y/N]: ") {
            print_info("Cancelled.");
            return Ok(());
        }

        let removed = SessionService::clear_sessions()?;
        print_success(format!("Cleared {} session(s).", removed));
        Ok(())
    }

    pub fn init(provider_type: ProviderType, languages: Vec<Language>) -> Result<()> {
        // 1. Check and handle existing flake.nix
        let current_dir = std::env::current_dir()?;
        let flake_path = current_dir.join("flake.nix");
        if flake_path.exists() {
            if is_terminal() {
                if !ask_confirmation("flake.nix already exists. Backup and overwrite? [y/N]: ") {
                    print_info("Cancelled.");
                    return Ok(());
                }
            } else {
                // Non-interactive mode: warn and auto-backup
                print_warning("flake.nix already exists. Auto-backing up to flake.nix.bak");
            }
            // Backup
            let backup_path = current_dir.join("flake.nix.bak");
            std::fs::copy(&flake_path, &backup_path)?;
            print_success(format!(
                "Backed up existing flake.nix to {}",
                backup_path.display()
            ));
        }

        // 2. Generate flake.nix content
        let supported = to_supported_languages(provider_type, &languages)?;
        let flake_contents = get_flake_contents(provider_type)(&supported)?;

        // 3. Write file
        std::fs::write(&flake_path, flake_contents)?;
        print_success(format!("Created {}", flake_path.display()));

        // 4. Execute nix develop
        print_bold("Entering develop shell...");
        let mut cmd = std::process::Command::new("nix");
        cmd.arg("develop").arg(&current_dir);
        if provider_type == ProviderType::Devenv {
            cmd.arg("--no-pure-eval");
        }
        let env_shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        cmd.arg("--command").arg(env_shell);
        exec(cmd)
    }

    pub fn list_provider() -> Result<()> {
        let providers = ProviderShowSelector::All.as_provider_types();
        print_provider_list(providers)?;
        Ok(())
    }

    pub fn list_sessions() -> Result<()> {
        let sessions = SessionService::list_sessions()?;
        if sessions.is_empty() {
            print_info("No sessions found.");
            return Ok(());
        }

        print_sessions_list(&sessions);
        Ok(())
    }

    pub fn remove_sessions(keys: &[SessionKey]) -> Result<()> {
        let Some(result) = SessionService::remove_sessions(keys)? else {
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
        let session = SessionService::resolve_session_dir(key)?;
        nix_develop_existing_session(session)
    }

    pub fn show_provider(provider: ProviderShowSelector) -> Result<()> {
        let providers = provider.as_provider_types();
        print_provider_show(providers)?;
        Ok(())
    }

    pub fn update_session(key: Option<&SessionKey>) -> Result<()> {
        // Resolve session: use provided key or fall back to current session
        let session = match key {
            Some(k) => SessionService::resolve_session_dir(k)?,
            None => {
                let current_id = crate::paths::read_current_session()?.ok_or_else(|| {
                    anyhow::anyhow!(
                        "No current session. Specify a session with 'ah update <index|id>'"
                    )
                })?;
                SessionService::resolve_session_dir(&SessionKey::Id(current_id))?
            }
        };

        let session_dir = session.get_dir()?;
        let lock_path = session_dir.join("flake.lock");

        // Get modification time before update
        let mtime_before = lock_path.metadata().and_then(|m| m.modified()).ok();

        print_bold("Updating flake dependencies...");

        // Run nix flake update in the session directory
        let mut cmd = std::process::Command::new("nix");
        cmd.arg("flake").arg("update").current_dir(&session_dir);

        let output = cmd.output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nix flake update failed: {}", err);
        }

        // Check if flake.lock was actually updated
        let mtime_after = lock_path.metadata().and_then(|m| m.modified()).ok();
        let was_updated = match (mtime_before, mtime_after) {
            (Some(before), Some(after)) => after > before,
            (None, Some(_)) => true,
            _ => false,
        };

        if was_updated {
            print_success("Dependencies updated.");

            // Prompt user to enter new development environment
            if is_terminal() {
                if ask_confirmation("Enter new development environment? [Y/n]: ") {
                    print_bold("Entering develop shell...");
                    nix_develop_existing_session(session)?;
                } else {
                    print_info("Skipped. Run 'ah session restore' to enter manually.");
                }
            }
        } else {
            print_info("Dependencies are already up to date.");
        }

        Ok(())
    }

    pub fn use_languages(provider_type: ProviderType, languages: Vec<Language>) -> Result<()> {
        match SessionService::find_session(provider_type, &languages)? {
            Some(session) => {
                print_session_found(
                    &session.id,
                    &session.provider.to_string(),
                    &session.languages,
                );
                print_bold("Restoring develop shell...");
                nix_develop_existing_session(session)
            }
            None => {
                let lang_strings: Vec<String> = languages.iter().map(|l| l.to_string()).collect();
                print_no_session(&provider_type.to_string(), &lang_strings);
                print_bold("Creating develop shell...");
                let session = SessionService::create_session(provider_type, languages)?;
                nix_develop_new_session(session)
            }
        }
    }
}
