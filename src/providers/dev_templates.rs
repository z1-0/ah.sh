use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_json::from_str;
use crate::{env, providers::EnvironmentProvider};

pub struct DevTemplatesProvider;

impl EnvironmentProvider for DevTemplatesProvider {
    fn name(&self) -> &str {
        "dev-templates"
    }

    fn get_dir(&self) -> PathBuf {
        Path::new(&env::providers_dir()).join("dev-templates")
    }

    fn get_supported_languages(&self) -> Vec<String> {
        let json_str = include_str!("../../providers/dev-templates/supported_langs.json");
        from_str(json_str).expect("Internal error")
    }

    fn normalize_language(&self, lang: &str) -> String {
        let aliases: HashMap<String, HashMap<String, String>> =
            from_str(include_str!("../assets/language_aliases.json")).expect("Internal error");

        aliases
            .get(lang)
            .and_then(|m| m.get("dev-templates"))
            .cloned()
            .unwrap_or_else(|| lang.to_owned())
    }
}
