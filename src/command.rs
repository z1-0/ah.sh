use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn exec_nix_develop(
    provider_flake_dir: &str,
    env_ahsh_languages: String,
    profile_path: Option<std::path::PathBuf>,
) {
    let mut cmd = Command::new("nix");
    cmd.args([
        "develop",
        "--no-pure-eval",
        &format!("path:{}", provider_flake_dir),
    ]);

    if let Some(path) = profile_path {
        cmd.arg("--profile");
        cmd.arg(path);
    }

    let _ = cmd.env("AHSH_LANGUAGES", env_ahsh_languages).exec();
}
