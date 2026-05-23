use assert_cmd::Command;
use predicates::prelude::*;

fn command() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn check_current_repository() {
    let mut cmd = command();

    cmd.arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository: ."))
        .stdout(predicate::str::contains("[PASS] readme"));
}

#[test]
fn check_outputs_json() {
    let mut cmd = command();

    cmd.args(["check", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"checks\""))
        .stdout(predicate::str::contains("\"id\": \"readme\""));
}

#[test]
fn check_can_fail_on_warnings() {
    let mut cmd = command();

    cmd.args(["check", "--fail-on", "warn"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("[WARN] code_of_conduct"));
}
