use crate::output::*;
use crate::provider::{Language, ProviderShowSelector, ProviderType};
use crate::session::SessionService;
use crate::session::{Session, SessionKey};
use anyhow::Result;

pub struct Manager;

fn nix_develop_new_session(session: Session) -> Result<()> {
    crate::cmd::nix_develop(session, false)
}

fn nix_develop_existing_session(session: Session) -> Result<()> {
    crate::cmd::nix_develop(session, true)
}

impl Manager {
    pub fn list_sessions() -> Result<()> {
        let sessions = SessionService::list_sessions()?;
        if sessions.is_empty() {
            print_info("No sessions found.");
            return Ok(());
        }

        print_sessions_list(sessions);
        Ok(())
    }

    pub fn restore_session(key: &SessionKey) -> Result<()> {
        let session = SessionService::resolve_session_dir(key)?;
        nix_develop_existing_session(session)
    }

    pub fn clear_sessions() -> Result<()> {
        if is_terminal() && !ask_confirmation("This will remove all sessions. Continue? [y/N]: ") {
            print_info("Cancelled.");
            return Ok(());
        }

        let removed = SessionService::clear_sessions()?;
        print_success(format!("Cleared {} session(s).", removed));
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

    pub fn list_provider() -> Result<()> {
        let providers = ProviderShowSelector::All.as_provider_types();
        print_provider_list(providers)?;
        Ok(())
    }

    pub fn show_provider(provider: ProviderShowSelector) -> Result<()> {
        let providers = provider.as_provider_types();
        print_provider_show(providers)?;
        Ok(())
    }
}
