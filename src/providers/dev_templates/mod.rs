pub mod fetcher;
pub mod flake_generator;
pub mod nix_parser;

use crate::error::Result;
use crate::providers::{ProviderAssetManager, ShellProvider};
use std::path::PathBuf;

pub struct DevTemplatesProvider {
    manager: ProviderAssetManager,
}

impl Default for DevTemplatesProvider {
    fn default() -> Self {
        Self {
            manager: ProviderAssetManager::new(
                "dev-templates",
                include_str!("../../assets/providers/dev-templates/flake.nix"),
                include_str!("../../assets/providers/dev-templates/supported_langs.json"),
            ),
        }
    }
}

impl ShellProvider for DevTemplatesProvider {
    fn name(&self) -> &str {
        "dev-templates"
    }

    fn ensure_files(&self, languages: &[String]) -> Result<PathBuf> {
        // We override ensure_files to dynamically generate the flake.nix based on requested languages
        let dir = crate::paths::get_xdg_dir(crate::paths::XdgDir::Data)?
            .join("providers")
            .join(self.name());
        std::fs::create_dir_all(&dir)?;

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

        let flake_path = dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(dir)
    }

    fn get_supported_languages(&self) -> Result<Vec<String>> {
        self.manager.get_supported_languages()
    }

    fn normalize_language(&self, lang: &str) -> String {
        self.manager.normalize_language(lang)
    }
}
