use anyhow::Result;

pub mod flake_generator;

pub fn get_flake_contents(languages: &[String]) -> Result<String> {
    Ok(flake_generator::generate_devenv_flake(languages))
}
