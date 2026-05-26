use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

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
    let temp_dir = tempdir().unwrap();

    cmd.args([
        "check",
        temp_dir.path().to_str().unwrap(),
        "--fail-on",
        "warn",
    ])
    .assert()
    .failure()
    .stdout(predicate::str::contains("[WARN] readme"));
}

#[test]
fn check_accepts_comma_separated_profiles() {
    let mut cmd = command();
    let temp_dir = tempdir().unwrap();

    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name":"demo","version":"0.1.0","description":"Demo","license":"MIT","repository":"https://example.com/demo","packageManager":"npm@11.0.0","scripts":{"test":"node --test"},"engines":{"node":">=20"}}"#,
    )
    .unwrap();

    cmd.args([
        "check",
        temp_dir.path().to_str().unwrap(),
        "--profiles",
        "rust,node",
        "--format",
        "compact",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("profiles=rust, node"));
}

#[test]
fn check_rejects_unknown_comma_separated_profile() {
    let mut cmd = command();

    cmd.args(["check", "--profiles", "rust,unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
