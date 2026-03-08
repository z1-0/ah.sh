use crate::error::{AppError, Result};
use crate::executor::execute_nix_develop;
use crate::providers::ProviderType;
use crate::sessions::{self, Session};
use std::collections::HashSet;

pub struct Manager;

impl Manager {
    pub fn list_sessions() -> Result<()> {
        let list = sessions::list_sessions()?;
        if list.is_empty() {
            println!("No sessions found.");
            return Ok(());
        }
        println!("{:<5} {:<10} {:<15} Languages", "ID", "Hash", "Provider");
        for (i, s) in list.iter().enumerate() {
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

    pub fn restore_session(args: &str) -> Result<()> {
        let session = sessions::find_session(args)?;
        let session_dir = sessions::get_session_dir()?.join(&session.id);
        execute_nix_develop(session_dir, false);
        Ok(())
    }

    pub fn create_session(provider_type: ProviderType, languages: Vec<String>) -> Result<()> {
        let provider = provider_type.into_shell_provider();

        // 1. Normalize and validate languages
        let mut normalized_langs = languages
            .iter()
            .map(|l| provider.normalize_language(l))
            .collect::<Vec<_>>();

        let mut seen = HashSet::new();
        normalized_langs.retain(|lang| seen.insert(lang.clone()));

        if normalized_langs.is_empty() {
            return Err(AppError::Generic(
                "No languages specified. Use 'ah <langs>' or 'ah session list'".to_string(),
            ));
        }

        let supported_langs = provider.get_supported_languages()?;
        Self::validate_languages(&normalized_langs, &supported_langs)?;

        // 2. Prepare Session and Directory
        let session_id = sessions::generate_id(provider.name(), &normalized_langs);
        let session_dir = sessions::get_session_dir()?.join(&session_id);
        std::fs::create_dir_all(&session_dir)?;

        // 3. Generate Flake in Session Directory
        provider.ensure_files(&normalized_langs, &session_dir)?;

        // 4. Session Metadata Management
        let session = Session::new(session_id, normalized_langs, provider.name().to_string());
        sessions::save_session(&session)?;

        // 5. Execute
        execute_nix_develop(session_dir, true);

        Ok(())
    }

    fn validate_languages(langs: &[String], supported: &[String]) -> Result<()> {
        let supported_set: HashSet<_> = supported.iter().collect();
        let invalids: Vec<_> = langs
            .iter()
            .filter(|l| !supported_set.contains(l))
            .cloned()
            .collect();

        if invalids.is_empty() {
            Ok(())
        } else {
            Err(AppError::UnsupportedLanguages(invalids))
        }
    }
}
