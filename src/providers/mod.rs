use std::path::PathBuf;

pub mod dev_templates;
pub mod devenv;

pub trait EnvironmentProvider {
    fn name(&self) -> &str;
    fn get_dir(&self) -> PathBuf;
    fn get_supported_languages(&self) -> Vec<String>;
    fn normalize_language(&self, lang: &str) -> String;
}
