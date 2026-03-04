pub mod fetcher;
pub mod flake_generator;
pub mod nix_parser;

use crate::error::Result;
use crate::providers::ShellProvider;
use std::path::Path;

pub struct DevTemplatesProvider;

impl Default for DevTemplatesProvider {
    fn default() -> Self {
        Self
    }
}

impl ShellProvider for DevTemplatesProvider {
    fn name(&self) -> &str {
        "dev-templates"
    }

    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<()> {
        let mut parsed_attrs = Vec::new();

        for lang in languages {
            // Check if it's the 'empty' template which basically does nothing
            if lang == "empty" {
                continue;
            }

            match self::fetcher::fetch_flake_source(lang) {
                Ok(source) => {
                    let attrs = self::nix_parser::parse_flake_shell(&source);
                    parsed_attrs.push((lang.clone(), attrs));
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to fetch flake source for '{}': {}",
                        lang, e
                    );
                    // Just continue, inputsFrom will still work for packages/shellHook
                }
            }
        }

        let flake_content =
            self::flake_generator::generate_dev_templates_flake(languages, &parsed_attrs);

        let flake_path = target_dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(())
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        let langs_json = include_str!("../../assets/providers/dev-templates/supported_langs.json");
        let langs = serde_json::from_str(langs_json)?;
        Ok(langs)
    }
}
