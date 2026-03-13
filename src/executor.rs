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

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    #[test]
    fn exec_missing_command_surfaces_not_found_as_io_error() {
        let err = super::exec_cmd(std::process::Command::new("__definitely_missing__"))
            .expect_err("expected exec to fail");

        let crate::error::AppError::Io(io_err) = err else {
            panic!("expected AppError::Io, got {err:?}");
        };

        assert_eq!(io_err.kind(), ErrorKind::NotFound);
    }
}
