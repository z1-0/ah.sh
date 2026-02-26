use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn exec_nix_develop(provider_flake_dir: &str, env_ahsh_languages: String) {
    let _ = Command::new("nix")
        .args([
            "develop",
            "--no-pure-eval",
            &format!("path:{}", provider_flake_dir),
        ])
        .env("AHSH_LANGUAGES", env_ahsh_languages)
        .exec();
}
