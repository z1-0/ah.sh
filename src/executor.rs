use crate::error::{AppError, Result};
use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn exec_cmd(mut cmd: Command) -> Result<Infallible> {
    let err = cmd.exec();
    Err(AppError::Io(err))
}

pub fn execute_nix_develop(session_dir: PathBuf, new_session: bool) -> Result<Infallible> {
    let profile_path = session_dir.join("nix-profile");

    let mut cmd = Command::new("nix");
    cmd.arg("develop");
    cmd.arg("--profile").arg(profile_path);

    if new_session {
        cmd.arg("--no-pure-eval").arg(&session_dir);
    }

    if cfg!(debug_assertions) {
        eprintln!("Executing: {:?}", cmd);
    }

    exec_cmd(cmd)
}
