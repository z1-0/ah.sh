use crate::providers::ShellProvider;
use serde_json::from_str;
use std::collections::HashMap;

pub struct DevenvProvider;

impl ShellProvider for DevenvProvider {
    fn name(&self) -> &str {
        "devenv"
    }

    fn ensure_files(&self) -> Result<std::path::PathBuf, String> {
        let dir = crate::env::get_ah_data_dir()
            .join("providers")
            .join("devenv");
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create provider dir: {e}"))?;

        let flake_path = dir.join("flake.nix");
        let flake_content = include_str!("../assets/providers/devenv/flake.nix");

        std::fs::write(flake_path, flake_content)
            .map_err(|e| format!("Failed to write flake.nix: {e}"))?;

        Ok(dir)
    }

    fn get_supported_languages(&self) -> Vec<String> {
        let json_str = include_str!("../assets/providers/devenv/supported_langs.json");
        from_str(json_str).expect("Internal error")
    }

    fn normalize_language(&self, lang: &str) -> String {
        let aliases: HashMap<String, HashMap<String, String>> =
            from_str(include_str!("../assets/language_aliases.json")).expect("Internal error");

        aliases
            .get(lang)
            .and_then(|m| m.get("devenv"))
            .cloned()
            .unwrap_or_else(|| lang.to_owned())
    }
}
