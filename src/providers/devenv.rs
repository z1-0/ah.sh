use crate::{env, providers::EnvironmentProvider};
use serde_json::from_str;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct DevenvProvider;

impl EnvironmentProvider for DevenvProvider {
    fn name(&self) -> &str {
        "devenv"
    }

    fn get_dir(&self) -> PathBuf {
        Path::new(&env::providers_dir()).join("devenv")
    }

    fn get_supported_languages(&self) -> Vec<String> {
        let json_str = include_str!("../../providers/devenv/supported_langs.json");
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
