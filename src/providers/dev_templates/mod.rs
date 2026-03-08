pub mod fetcher;
pub mod flake_generator;
pub mod nix_parser;

use crate::error::{AppError, Result};
use crate::providers::ShellProvider;
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;
use std::thread;

pub struct DevTemplatesProvider;

static SUPPORTED_LANGUAGES: OnceLock<std::result::Result<Vec<String>, String>> = OnceLock::new();
const MAX_FETCH_CONCURRENCY: usize = 8;

impl ShellProvider for DevTemplatesProvider {
    fn name(&self) -> &str {
        "dev-templates"
    }

    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<()> {
        let mut seen = HashSet::new();
        let deduped_languages: Vec<String> = languages
            .iter()
            .filter(|lang| seen.insert((*lang).clone()))
            .cloned()
            .collect();
        let fetch_languages: Vec<String> = deduped_languages
            .iter()
            .filter(|lang| *lang != "empty")
            .cloned()
            .collect();
        let fetch_concurrency = thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(4)
            .clamp(1, MAX_FETCH_CONCURRENCY);

        let mut parsed_attrs = Vec::new();

        for chunk in fetch_languages.chunks(fetch_concurrency) {
            let mut fetch_tasks = Vec::with_capacity(chunk.len());

            for lang in chunk {
                let lang = lang.clone();
                let task = thread::spawn(move || {
                    let result = self::fetcher::fetch_flake_source(&lang)
                        .map(|source| self::nix_parser::parse_flake_shell(&source));
                    (lang, result)
                });
                fetch_tasks.push(task);
            }

            for task in fetch_tasks {
                match task.join() {
                    Ok((lang, Ok(attrs))) => parsed_attrs.push((lang, attrs)),
                    Ok((lang, Err(e))) => {
                        eprintln!(
                            "Warning: Failed to fetch flake source for '{}': {}",
                            lang, e
                        );
                        // Just continue, inputsFrom will still work for packages/shellHook
                    }
                    Err(_) => {
                        eprintln!("Warning: Flake fetch task panicked unexpectedly");
                    }
                }
            }
        }

        let flake_content =
            self::flake_generator::generate_dev_templates_flake(&deduped_languages, &parsed_attrs);

        let flake_path = target_dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(())
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        let langs = SUPPORTED_LANGUAGES.get_or_init(|| {
            let langs_json =
                include_str!("../../assets/providers/dev-templates/supported_langs.json");
            serde_json::from_str(langs_json).map_err(|e| e.to_string())
        });

        langs.clone().map_err(AppError::Provider)
    }
}
