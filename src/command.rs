use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn exec_nix_develop(
    provider_flake_dir: &str,
    env_ahsh_languages: String,
    env_ahsh_packages: String,
) {
    Command::new("nix")
        .args(["develop", "--no-pure-eval", &format!("path:{}", provider_flake_dir)])
        .env("AHSH_LANGUAGES", env_ahsh_languages)
        .env("AHSH_PACKAGES", env_ahsh_packages)
        .exec();
}
