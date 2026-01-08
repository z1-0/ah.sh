use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::Parser;
use serde_json::from_str;

use crate::{command::exec_nix_develop, env};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,
}

pub fn languages() -> Result<(), String> {
    let cli = Cli::parse();

    let supported = supported_langs_of_devenv()?;
    let ensures = ensure_languages(cli.language, supported)?;

    let pkgs = flatten_pkgs(&ensures, query_pkgs_of_supported_langs());

    let env_ahsh_languages =
        serde_json::to_string(&ensures).expect("Failed to serialize languages");
    let env_ahsh_packages = serde_json::to_string(&pkgs).expect("Failed to serialize packages");

    exec_nix_develop(env_ahsh_languages, env_ahsh_packages);
    Ok(())
}

fn supported_langs_of_devenv() -> Result<Vec<String>, String> {
    let lang_path = Path::new(env::AHSH_DEVENV_SRC).join("src/modules/languages");

    Ok(lang_path
        .read_dir()
        .map_err(|e| format!("Failed to read directory {}: {}", lang_path.display(), e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = if path.extension().map_or(false, |ext| ext == "nix") {
                path.file_stem()?
            } else {
                path.file_name()?
            };
            name.to_str().map(|s| s.to_owned())
        })
        .collect::<Vec<_>>())
}

fn ensure_languages(
    langs: Vec<String>,
    supported_langs: Vec<String>,
) -> Result<Vec<String>, String> {
    let supported: HashSet<String> = supported_langs.into_iter().collect();

    let invalids: Vec<String> = langs
        .iter()
        .filter(|&l| !supported.contains(l))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(langs)
    } else {
        Err(format!("Languages {:?} are not supported", invalids))
    }
}

fn query_pkgs_of_supported_langs() -> HashMap<String, Vec<String>> {
    let json_str = include_str!("./assets/lang_pkgs.json");
    from_str(json_str).expect("Internal error")
}

fn flatten_pkgs(ensures: &[String], pkgs: HashMap<String, Vec<String>>) -> Vec<String> {
    ensures
        .iter()
        .filter_map(|x| pkgs.get(x))
        .flatten()
        .cloned()
        .collect()
}
