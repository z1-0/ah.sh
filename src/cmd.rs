use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::{paths::save_current_session, provider::ProviderType, session::Session};

pub fn nix_develop(session: Session, use_existing_profile: bool) -> Result<()> {
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

pub fn exec(mut cmd: Command) -> Result<()> {
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
