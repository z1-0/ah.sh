pub mod dev_templates;
pub mod devenv;

use std::path::PathBuf;

pub trait ShellProvider {
    fn name(&self) -> &str;
    fn ensure_files(&self) -> Result<PathBuf, String>;
    fn get_supported_languages(&self) -> Vec<String>;
    fn normalize_language(&self, lang: &str) -> String;
}
