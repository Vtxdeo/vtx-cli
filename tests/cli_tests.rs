use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("vtx").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Build and package plugins"));
}

#[test]
fn test_build_missing_config() {
    let mut cmd = Command::cargo_bin("vtx").unwrap();

    cmd.arg("build")
        .assert()
        .failure()
        .stderr(contains("Configuration file 'vtx.toml' not found"));
}
