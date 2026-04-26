use anyhow::Result;
use tracing_attributes::instrument;

use crate::provider::devenv::flake_generator::generate_devenv_flake;

pub mod flake_generator;

#[instrument(skip_all, fields(provider = "devenv"))]
pub fn get_flake_contents(languages: &[String]) -> Result<String> {
    Ok(generate_devenv_flake(languages))
}
