use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn help_exits_zero() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.arg("--help").assert().success().code(0);
}

#[test]
fn no_args_prints_help_and_exits_2() {
    // CLI contract:
    // - running `ah` with no args is a *usage* error (exit code 2)
    // - it must still print the clap help to *stdout* (not stderr)
    let mut cmd = Command::cargo_bin("ah").unwrap();

    let usage_stdout = predicate::str::contains("USAGE").or(predicate::str::contains("Usage"));
    let usage_stderr = predicate::str::contains("USAGE").or(predicate::str::contains("Usage"));

    cmd.assert()
        .failure()
        .code(2)
        // Help must be on stdout.
        .stdout(usage_stdout.and(predicate::str::contains("session")))
        // Help should not be printed to stderr.
        .stderr(usage_stderr.not());
}

#[test]
fn session_list_empty_is_ok() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.env("XDG_CACHE_HOME", tmp.path())
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No sessions found."));
}

#[test]
fn session_clear_non_tty_does_not_block() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.env("XDG_CACHE_HOME", tmp.path())
        .args(["session", "clear"])
        .write_stdin("")
        .timeout(Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared"));
}
