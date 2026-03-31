use anyhow::{Context, Result};
use std::process::Command;
use std::{os::unix::process::CommandExt, path::PathBuf};

use crate::{paths::save_current_session, provider::ProviderType, session::Session};

pub fn nix_develop_of_path(provider: ProviderType, flake_url: PathBuf) -> Result<()> {
    let mut cmd = Command::new("nix");
    cmd.arg("develop").arg(&flake_url);

    build_nix_develop_cmd(&mut cmd, provider);
    exec(cmd)
}

pub fn nix_develop_of_session(session: Session, use_existing_profile: bool) -> Result<()> {
    let flake_dir = session.get_dir()?;
    let profile_path = flake_dir.join("nix-profile");

    // Record current session before entering
    save_current_session(&session.id)?;

    let mut cmd = Command::new("nix");
    cmd.arg("develop");

    if use_existing_profile {
        cmd.arg(&profile_path);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(&profile_path);
    }

    build_nix_develop_cmd(&mut cmd, session.provider);
    exec(cmd)
}

/// Common setup for nix develop commands: devenv flags and shell configuration
fn build_nix_develop_cmd(cmd: &mut Command, provider: ProviderType) {
    if provider == ProviderType::Devenv {
        cmd.arg("--no-pure-eval");
    }

    let env_shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    cmd.arg("--command").arg(env_shell);
}

pub fn nix_flake_update_of_session(session: &Session) -> Result<String> {
    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("update")
        .current_dir(session.get_dir()?);

    run(cmd)
}

pub fn prefetch_dev_templates() -> Result<String> {
    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("prefetch")
        .arg("--json")
        .arg("github:the-nix-way/dev-templates");

    run(cmd)
}

fn run(mut cmd: Command) -> Result<String> {
    let command = command_to_string(&cmd);
    let output = cmd
        .output()
        .context(format!("failed to start command: {}", command))?;

    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = if details.is_empty() {
            format!("exit status {}", output.status)
        } else {
            details
        };

        anyhow::bail!("command `{}` failed: {}", command, details);
    }

    String::from_utf8(output.stdout).context(format!("invalid UTF-8 output from `{}`", command))
}

fn exec(mut cmd: Command) -> Result<()> {
    let command = command_to_string(&cmd);

    // Only print command in debug mode
    #[cfg(debug_assertions)]
    println!("{command}");

    let source = cmd.exec();
    anyhow::bail!("failed to exec: {}: {}", command, source)
}

fn command_to_string(cmd: &Command) -> String {
    let mut parts = vec![cmd.get_program().to_string_lossy().into_owned()];
    parts.extend(
        cmd.get_args()
            .map(|value| value.to_string_lossy().into_owned()),
    );
    parts.join(" ")
}
