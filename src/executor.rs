use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub fn execute_nix_develop(session_dir: PathBuf, new_session: bool) {
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

    let _ = cmd.exec();
}
