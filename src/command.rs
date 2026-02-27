use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub fn exec_nix_develop_with_provider(
    provider_flake_dir: &str,
    env_ahsh_languages: String,
    session_src: PathBuf,
) {
    Command::new("nix")
        .args([
            "develop",
            "--no-pure-eval",
            &format!("path:{}", provider_flake_dir),
            "--profile",
        ])
        .arg(session_src)
        .env("AHSH_LANGUAGES", env_ahsh_languages)
        .exec();
}

pub fn exec_nix_develop_with_session(session_src: PathBuf) {
    Command::new("nix")
        .args(["develop", "--profile"])
        .arg(session_src)
        .exec();
}
