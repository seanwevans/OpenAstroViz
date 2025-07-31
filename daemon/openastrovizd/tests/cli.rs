use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

#[test]
fn runs_without_args_shows_version() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn status_subcommand() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon status"));
}

#[test]
fn bench_cuda_subcommand() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.args(["bench", "cuda"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cuda"));
}

#[test]
fn help_includes_description() {
    Command::cargo_bin("openastrovizd")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("OpenAstroViz daemon"));
}

#[test]
fn start_subcommand_outputs_message() {
    Command::cargo_bin("openastrovizd")
        .unwrap()
        .arg("start")
        .assert()
        .success()
        .stdout(contains("Starting daemon"));
}
