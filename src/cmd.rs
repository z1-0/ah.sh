use crate::path;
use crate::provider::ProviderType;
use crate::session::Session;
use anyhow::Context;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn check_nix_available() -> anyhow::Result<()> {
    Command::new("nix")
        .arg("--version")
        .output()
        .map(|_| ())
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "Nix is not installed.\n\n\
                     Install with the Determinate Nix Installer:\n  \
                     curl -fsSL https://install.determinate.systems/nix | sh -s -- install"
                )
            } else {
                anyhow::Error::from(e).context("failed to check Nix availability")
            }
        })
}

pub fn nix_develop_of_session(session: Session) -> anyhow::Result<()> {
    check_nix_available()?;

    let flake_dir = session.get_dir();
    let profile_file = flake_dir.join(path::cache::sessions::NIX_PROFILE_FILE);

    path::cache::save_current_session(&session.id)?;

    let cwd = path::get_cwd()?;
    if let Err(e) = crate::session::update_history(&session, &cwd) {
        eprintln!("Warning: failed to update session history: {}", e);
    }

    let mut cmd = Command::new("nix");
    cmd.arg("develop");

    if profile_file.exists() {
        cmd.arg(&profile_file);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(&profile_file);
    }

    if session.provider == ProviderType::Devenv {
        cmd.arg("--no-pure-eval");
    }

    if let Some(shell) = crate::util::get_shell() {
        cmd.arg("--command").arg(shell);
    }

    let err = cmd.exec();
    anyhow::bail!("failed to execute nix develop: {err}")
}

pub fn nix_flake_update_of_session(session: &Session) -> anyhow::Result<String> {
    check_nix_available()?;

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("update")
        .current_dir(session.get_dir());

    let output = cmd.output().context("failed to run nix flake update")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    String::from_utf8(output.stdout).context("failed to decode nix output")
}

pub fn prefetch_dev_templates() -> anyhow::Result<String> {
    check_nix_available()?;

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("prefetch")
        .arg("--json")
        .arg("github:the-nix-way/dev-templates");

    let output = cmd.output().context("failed to run nix flake prefetch")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    String::from_utf8(output.stdout).context("failed to decode nix output")
}
