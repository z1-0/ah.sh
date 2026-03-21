use crate::error::{AppError, Result};
use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Failed to start command `{command}`: {source}")]
    Io {
        command: String,
        source: std::io::Error,
    },

    #[error("Command `{command}` failed: {details}")]
    Failed { command: String, details: String },
}

fn run(mut cmd: Command) -> Result<String> {
    let command = command_to_string(&cmd);
    let output = cmd.output().map_err(|source| CommandError::Io {
        command: command.clone(),
        source,
    })?;

    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = if details.is_empty() {
            format!("exit status {}", output.status)
        } else {
            details
        };

        return Err(CommandError::Failed { command, details }.into());
    }

    let command_for_decode_error = command.clone();
    String::from_utf8(output.stdout).map_err(|err| {
        AppError::Generic(format!(
            "invalid UTF-8 output from `{command_for_decode_error}`: {err}"
        ))
    })
}

fn exec(mut cmd: Command) -> Result<Infallible> {
    if cfg!(debug_assertions) {
        eprintln!("exec: {}", command_to_string(&cmd));
    }

    let command = command_to_string(&cmd);
    let source = cmd.exec();
    Err(CommandError::Io { command, source }.into())
}

pub fn nix_develop(flake_dir: PathBuf, use_profile: bool) -> Result<Infallible> {
    let profile_path = flake_dir.join("nix-profile");

    let mut cmd = Command::new("nix");
    cmd.arg("develop").arg("--no-pure-eval");

    if use_profile {
        cmd.arg(profile_path);
    } else {
        cmd.arg(&flake_dir).arg("--profile").arg(profile_path);
    }

    exec(cmd)
}

pub fn nix_flake_prefetch(lang: &str) -> Result<String> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");

    let mut cmd = Command::new("nix");
    cmd.arg("flake")
        .arg("prefetch")
        .arg("--json")
        .arg(&flake_ref);

    run(cmd)
}

fn command_to_string(cmd: &Command) -> String {
    let mut parts = vec![cmd.get_program().to_string_lossy().into_owned()];
    parts.extend(
        cmd.get_args()
            .map(|value| value.to_string_lossy().into_owned()),
    );
    parts.join(" ")
}
