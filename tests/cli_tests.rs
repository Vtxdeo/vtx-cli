use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vtx"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Build and package the plugin"));
}

#[test]
fn test_build_missing_config() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vtx"));
    cmd.arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unable to resolve package name"));
}
