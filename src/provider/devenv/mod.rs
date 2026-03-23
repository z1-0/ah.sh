pub mod flake_generator;

use anyhow::Result;
use std::path::Path;

pub fn ensure_files(languages: &[String], target_dir: &Path) -> Result<()> {
    let flake_content = self::flake_generator::generate_devenv_flake(languages);

    let flake_path = target_dir.join("flake.nix");
    std::fs::write(flake_path, flake_content)?;

    Ok(())
}
