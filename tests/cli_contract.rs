use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn help_exits_zero() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.arg("--help").assert().success().code(0);
}

#[test]
fn no_args_prints_help_and_exits_0() {
    // CLI contract:
    // - running `ah` with no args prints help (exit code 0)
    // - it must print the clap help to *stdout* (not stderr)
    let mut cmd = Command::cargo_bin("ah").unwrap();

    let usage_stdout = predicate::str::contains("USAGE").or(predicate::str::contains("Usage"));
    let usage_stderr = predicate::str::contains("USAGE").or(predicate::str::contains("Usage"));

    cmd.assert()
        .success()
        .code(0)
        // Help must be on stdout.
        .stdout(usage_stdout.and(predicate::str::contains("session")))
        // Help should not be printed to stderr.
        .stderr(usage_stderr.not());
}

#[test]
fn provider_without_subcommand_prints_provider_help_and_exits_0() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider"]).assert().success().code(0).stdout(
        predicate::str::contains("USAGE")
            .or(predicate::str::contains("Usage"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("show")),
    );
}

#[test]
fn provider_list_prints_provider_names() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devenv").and(predicate::str::contains("dev-templates")));
}

#[test]
fn provider_show_all_prints_both_provider_sections() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "show", "all"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Provider: devenv")
                .and(predicate::str::contains("Provider: dev-templates")),
        );
}

#[test]
fn provider_show_devenv_is_line_oriented_without_table() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "show", "devenv"])
        .assert()
        .success()
        // Should not include the previous table header.
        .stdout(predicate::str::contains("Language").not())
        // Should include at least one known language.
        .stdout(predicate::str::contains("rust"));
}

#[test]
fn provider_show_includes_aliases_in_parentheses() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "show", "devenv"])
        .assert()
        .success()
        // From language_aliases.json: cpp/c++ -> cplusplus for devenv.
        .stdout(predicate::str::contains("cplusplus(c++"));
}

#[test]
fn provider_help_mentions_show_all_examples() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show all"));
}

#[test]
fn provider_show_help_mentions_all() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["provider", "show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("all"));
}

#[test]
fn create_requires_subcommand() {
    // Ensure the implicit "ah <langs>" behavior is gone.
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["rust"]).assert().failure();
}

#[test]
fn provider_rejects_provider_flag() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.args(["-p", "devenv", "provider", "list"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("not supported").or(predicate::str::contains("provider")));
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
