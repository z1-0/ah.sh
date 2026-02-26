use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::{Parser, ValueEnum};
use serde_json::from_str;

use crate::{command::exec_nix_develop, env};

#[derive(ValueEnum, Clone)]
pub enum Provider {
    Devenv,
    DevTemplates,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,

    #[arg(long, value_enum, default_value = "devenv")]
    pub provider: Provider,
}

pub fn languages() -> Result<(), String> {
    let cli = Cli::parse();

    let provider_dir = provider_dir(&cli.provider);

    let normalized_langs = normalize_languages(cli.language, &cli.provider);

    let ensures = match cli.provider {
        Provider::Devenv => {
            let supported = supported_langs_of_devenv();
            ensure_languages(normalized_langs, supported)?
        }
        Provider::DevTemplates => {
            let supported = supported_langs_of_dev_templates();
            ensure_languages(normalized_langs, supported)?
        }
    };

    let pkgs = match cli.provider {
        Provider::Devenv => flatten_pkgs(&ensures, query_pkgs_of_supported_langs()),
        Provider::DevTemplates => vec![],
    };

    let env_ahsh_languages =
        serde_json::to_string(&ensures).expect("Failed to serialize languages");
    let env_ahsh_packages = serde_json::to_string(&pkgs).expect("Failed to serialize packages");

    exec_nix_develop(&provider_dir, env_ahsh_languages, env_ahsh_packages);
    Ok(())
}

fn provider_dir(provider: &Provider) -> String {
    let base = Path::new(env::AHSH_PROVIDERS_DIR);
    let subdir = match provider {
        Provider::Devenv => "devenv",
        Provider::DevTemplates => "dev-templates",
    };
    base.join(subdir)
        .to_str()
        .expect("Invalid provider path")
        .to_owned()
}

fn normalize_languages(langs: Vec<String>, provider: &Provider) -> Vec<String> {
    let aliases: HashMap<String, HashMap<String, String>> =
        from_str(include_str!("./assets/language_aliases.json")).expect("Internal error");

    let provider_key = match provider {
        Provider::Devenv => "devenv",
        Provider::DevTemplates => "dev-templates",
    };

    langs
        .into_iter()
        .map(|lang| {
            aliases
                .get(&lang)
                .and_then(|m| m.get(provider_key))
                .cloned()
                .unwrap_or(lang)
        })
        .collect()
}

fn supported_langs_of_devenv() -> Vec<String> {
    let json_str = include_str!("../providers/devenv/supported_langs.json");
    from_str(json_str).expect("Internal error")
}

fn supported_langs_of_dev_templates() -> Vec<String> {
    let json_str = include_str!("../providers/dev-templates/supported_langs.json");
    from_str(json_str).expect("Internal error")
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
