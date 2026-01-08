use std::{os::unix::process::CommandExt, process::Command};

const FLAKE_URL: &str = "github:6iovan/ah.sh#ah";

#[allow(unused_must_use)]
pub fn exec_nix_develop(env_ahsh_languages: String, env_ahsh_packages: String) {
    Command::new("nix")
        .args(["develop", "--no-pure-eval", FLAKE_URL])
        .env("AHSH_LANGUAGES", env_ahsh_languages)
        .env("AHSH_PACKAGES", env_ahsh_packages)
        .exec();
}
