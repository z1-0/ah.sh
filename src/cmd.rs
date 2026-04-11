use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::{
    paths::{get_cwd, save_current_session, session::NIX_PROFILE_FILE},
    provider::ProviderType,
    session::Session,
};

pub fn nix_develop_of_session(session: Session) -> Result<()> {
    let flake_dir = session.get_dir()?;
    let profile_file = flake_dir.join(NIX_PROFILE_FILE);

    // Record current session before entering
    save_current_session(&session.id)?;

    // Update session history with current working directory
    let cwd = get_cwd()?;
    if let Err(e) = crate::session::update_history(&session, &cwd) {
        eprintln!("Warning: failed to update session history: {}", e);
    }

    let mut cmd = Command::new("nix");
    cmd.arg("develop");

    // If profile file exists, restore from it directly
    // Otherwise, record environment to profile
    if profile_file.exists() {
        cmd.arg(&profile_file);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(&profile_file);
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
