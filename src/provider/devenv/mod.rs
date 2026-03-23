pub mod flake_generator;

use crate::provider::{EnsureFilesResult, ShellProvider};
use anyhow::Result;
use std::path::Path;

pub struct DevenvProvider;

impl ShellProvider for DevenvProvider {
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult> {
        let flake_content = self::flake_generator::generate_devenv_flake(languages);

        let flake_path = target_dir.join("flake.nix");
        std::fs::write(flake_path, flake_content)?;

        Ok(EnsureFilesResult {
            warnings: Vec::new(),
        })
    }
}
