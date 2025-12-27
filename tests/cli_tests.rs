use assert_cmd::prelude::*;
use std::process::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("vtx").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("构建并打包插件"));
}

#[test]
fn test_build_missing_config() {
    let mut cmd = Command::cargo_bin("vtx").unwrap();
    cmd.arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unable to resolve package name"));
}
