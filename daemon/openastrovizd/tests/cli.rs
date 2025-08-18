use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
#[cfg(not(unix))]
use std::process::Command as StdCommand;
use std::sync::Mutex;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn cleanup() {
    let pid_path = std::env::temp_dir().join("openastrovizd.pid");
    if let Ok(pid_str) = fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            #[cfg(unix)]
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
            #[cfg(not(unix))]
            let _ = StdCommand::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status();
        }
    }
    let _ = fs::remove_file(pid_path);
}

#[test]
fn runs_without_args_shows_version() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn status_subcommand() {
    let _lock = TEST_MUTEX.lock().unwrap();
    cleanup();
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon is not running"));
}

#[test]
fn bench_cuda_supported() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.args(["bench", "cuda"]).assert().success();
}

#[test]
fn bench_cpu_supported() {
    let mut cmd = Command::cargo_bin("openastrovizd").unwrap();
    cmd.args(["bench", "cpu"]).assert().success();
}

#[test]
fn bench_help_lists_backends() {
    Command::cargo_bin("openastrovizd")
        .unwrap()
        .args(["bench", "--help"])
        .assert()
        .success()
        .stdout(contains("cpu").and(contains("cuda")));
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
    let _lock = TEST_MUTEX.lock().unwrap();
    cleanup();
    Command::cargo_bin("openastrovizd")
        .unwrap()
        .arg("start")
        .assert()
        .success()
        .stdout(contains("Daemon started"));
    cleanup();
}
