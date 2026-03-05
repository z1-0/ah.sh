use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub fn exec_nix_develop_with_provider(session_dir: PathBuf) {
    let mut cmd = Command::new("nix");
    let profile_path = session_dir.join("nix-profile");

    cmd.arg("develop")
        .arg("--no-pure-eval")
        .arg(session_dir)
        .arg("--profile")
        .arg(profile_path);

    if cfg!(debug_assertions) {
        eprintln!("Executing: {:?}", cmd);
    }

    let _ = cmd.exec();
}

pub fn exec_nix_develop_with_session(session_dir: PathBuf) {
    let mut cmd = Command::new("nix");
    let profile_path = session_dir.join("nix-profile");

    cmd.arg("develop").arg("--profile").arg(profile_path);

    if cfg!(debug_assertions) {
        eprintln!("Executing: {:?}", cmd);
    }

    let _ = cmd.exec();
}
