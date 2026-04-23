use crate::provider::ProviderType;
use crate::session::Session;
use crate::{log, path};
use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::debug;
use tracing::span::Span;
use tracing_attributes::instrument;

fn check_nix_available() -> Result<()> {
    match Command::new("nix").arg("--version").output() {
        Ok(output) if output.status.success() => {
            debug!(nix_version = %String::from_utf8_lossy(&output.stdout));
            Ok(())
        }
        _ => {
            anyhow::bail!(
                "Nix not found.\n\n\
                 To install, use the Determinate Nix Installer:\n  \
                 curl -fsSL https://install.determinate.systems/nix | sh -s -- install"
            );
        }
    }
}

#[instrument(skip_all, err, fields(session_id = %session.id))]
pub fn nix_develop_of_session(session: Session) -> Result<()> {
    check_nix_available()?;

    let flake_dir = session.get_dir();
    let profile_file = flake_dir.join(path::cache::sessions::NIX_PROFILE_FILE);

    path::cache::save_current_session(&session.id)?;

    let cwd = path::get_cwd()?;

    crate::session::update_history(&session, &cwd)?;

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

    Span::current().record("cmd", format!("{:?}", cmd));

    log::shutdown();

    let err = cmd.exec();
    anyhow::bail!("failed to execute nix develop: {err}")
}

#[instrument(skip_all, err, fields(session_id = %session.id))]
pub fn nix_flake_update_of_session(session: &Session) -> Result<String> {
    check_nix_available()?;

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("update")
        .current_dir(session.get_dir());

    Span::current().record("cmd", format!("{:?}", cmd));

    let output = cmd.output().context("failed to run nix flake update")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    String::from_utf8(output.stdout).context("failed to decode nix output")
}

#[instrument(skip_all, err)]
pub fn prefetch_dev_templates() -> Result<String> {
    check_nix_available()?;

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("prefetch")
        .arg("--json")
        .arg("github:the-nix-way/dev-templates");

    Span::current().record("cmd", format!("{:?}", cmd));

    let output = cmd.output().context("failed to run nix flake prefetch")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    String::from_utf8(output.stdout).context("failed to decode nix output")
}
