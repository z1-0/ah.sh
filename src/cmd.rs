use crate::provider::ProviderType;
use crate::session::Session;
use crate::{log, path};
use anyhow::{Context, Result};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::{debug, error, info, warn};

fn check_nix_available() -> Result<()> {
    match Command::new("nix").arg("--version").output() {
        Ok(output) if output.status.success() => {
            debug!(target: "ah::cmd", "Nix available");
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            anyhow::bail!(
                "Nix is not installed.\n\n\
Install with the Determinate Nix Installer:\n\
  curl -fsSL https://install.determinate.systems/nix | sh -s -- install"
            );
        }
        Err(e) => Err(anyhow::Error::from(e).context("failed to check Nix availability")),
    }
}

pub fn nix_develop_of_session(session: Session) -> Result<()> {
    check_nix_available()?;

    let flake_dir = session.get_dir();
    let profile_file = flake_dir.join(path::cache::sessions::NIX_PROFILE_FILE);

    path::cache::save_current_session(&session.id)?;

    let cwd = path::get_cwd()?;
    if let Err(e) = crate::session::update_history(&session, &cwd) {
        warn!(target: "ah::cmd", session_id = %session.id, error = %e, "Failed to update session history");
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

    info!(
        target: "ah::cmd",
        session_id = %session.id,
        provider = %session.provider,
        "Starting nix develop"
    );

    log::shutdown();

    let err = cmd.exec();
    error!(
        target: "ah::cmd",
        session_id = %session.id,
        exit_code = 1,
        error = %err,
        "nix develop failed"
    );
    anyhow::bail!("failed to execute nix develop: {err}")
}

pub fn nix_flake_update_of_session(session: &Session) -> Result<String> {
    check_nix_available()?;

    info!(target: "ah::cmd", session_id = %session.id, "Starting nix flake update");

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("update")
        .current_dir(session.get_dir());

    let output = cmd.output().context("failed to run nix flake update")?;
    if !output.status.success() {
        error!(target: "ah::cmd", session_id = %session.id, "nix flake update failed");
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    info!(target: "ah::cmd", session_id = %session.id, "nix flake update completed");

    String::from_utf8(output.stdout).context("failed to decode nix output")
}

pub fn prefetch_dev_templates() -> Result<String> {
    info!(target: "ah::cmd", "Starting dev-templates prefetch");

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("prefetch")
        .arg("--json")
        .arg("github:the-nix-way/dev-templates");

    let output = cmd.output().context("failed to run nix flake prefetch")?;
    if !output.status.success() {
        error!(target: "ah::cmd", "dev-templates prefetch failed");
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    info!(target: "ah::cmd", "dev-templates prefetch completed");

    String::from_utf8(output.stdout).context("failed to decode nix output")
}
