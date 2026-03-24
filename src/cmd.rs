use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::{provider::ProviderType, session::Session};

pub fn nix_develop(session: Session, use_profile: bool) -> Result<()> {
    let flake_dir = session.get_dir()?;
    let profile_path = flake_dir.join("nix-profile");

    let mut cmd = Command::new("nix");
    cmd.arg("develop");

    if use_profile {
        cmd.arg(profile_path);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(profile_path);
    }

    if session.provider == ProviderType::Devenv {
        cmd.arg("--no-pure-eval");
    }

    let env_shell = std::env::var("SHELL")?;
    cmd.arg("--command").arg(env_shell);

    exec(cmd)
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

    let command_for_decode_error = command.clone();
    String::from_utf8(output.stdout).context(format!(
        "invalid UTF-8 output from `{}`",
        command_for_decode_error
    ))
}

fn exec(mut cmd: Command) -> Result<()> {
    if cfg!(debug_assertions) {
        tracing::debug!(exec = %command_to_string(&cmd), "executing command");
    }

    let command = command_to_string(&cmd);
    let source = cmd.exec();
    Err(anyhow::anyhow!("failed to exec: {}: {}", command, source))
}

fn command_to_string(cmd: &Command) -> String {
    let mut parts = vec![cmd.get_program().to_string_lossy().into_owned()];
    parts.extend(
        cmd.get_args()
            .map(|value| value.to_string_lossy().into_owned()),
    );
    parts.join(" ")
}
