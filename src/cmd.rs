use crate::error::{AppError, Result};
use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn run(mut cmd: Command) -> Result<Infallible> {
    if cfg!(debug_assertions) {
        eprintln!(
            "exec: {} {}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    let err = cmd.exec();
    Err(AppError::Io(err))
}

pub fn cmd_nix_develop(flake_dir: PathBuf, use_profile: bool) -> Result<Infallible> {
    let profile_path = flake_dir.join("nix-profile");

    let mut cmd = Command::new("nix");
    cmd.arg("develop").arg("--no-pure-eval");

    if use_profile {
        cmd.arg(profile_path);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(profile_path);
    }

    run(cmd)
}
