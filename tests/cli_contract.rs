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
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("USAGE").or(predicate::str::contains("Usage")));
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
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared"));
}
