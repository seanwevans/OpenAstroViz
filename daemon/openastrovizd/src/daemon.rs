use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn pid_file() -> PathBuf {
    env::temp_dir().join("openastrovizd.pid")
}

/// Starts the OpenAstroViz daemon by spawning a background process.
///
/// The command used can be overridden with the `OPENASTROVIZD_DAEMON_CMD`
/// environment variable (defaults to `sleep`). The optional argument for the
/// command can be set via `OPENASTROVIZD_DAEMON_ARG` (defaults to `60`).
/// A PID file is written to the system temporary directory so that the daemon
/// can later be queried.
pub fn start_daemon() -> Result<String, io::Error> {
    let cmd = env::var("OPENASTROVIZD_DAEMON_CMD").unwrap_or_else(|_| "sleep".to_string());
    let arg = env::var("OPENASTROVIZD_DAEMON_ARG").unwrap_or_else(|_| "60".to_string());

    let child = Command::new(&cmd)
        .arg(&arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let pid = child.id();
    fs::write(pid_file(), pid.to_string())?;

    Ok(format!("Daemon started with pid {pid}"))
}

#[cfg(unix)]
fn process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn process_running(pid: u32) -> bool {
    let filter = format!("PID eq {pid}");
    if let Ok(out) = Command::new("tasklist")
        .args(["/FI", &filter, "/NH"])
        .output()
    {
        if !out.status.success() {
            return false;
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<_> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
        lines.len() == 1 && lines[0].contains(&pid.to_string())
    } else {
        false
    }
}

/// Checks the status of the OpenAstroViz daemon by reading the PID file and
/// verifying that the process is still alive.
pub fn check_status() -> Result<String, io::Error> {
    match fs::read_to_string(pid_file()) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if process_running(pid) {
                    return Ok(format!("Daemon is running with pid {pid}"));
                }
            }
            Ok(String::from("Daemon is not running"))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::from("Daemon is not running")),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn cleanup() {
        let pid_path = pid_file();
        if let Ok(pid_str) = fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                #[cfg(unix)]
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
                #[cfg(not(unix))]
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .status();
            }
        }
        let _ = fs::remove_file(pid_path);
    }

    #[test]
    fn start_and_status_success() {
        let _lock = TEST_MUTEX.lock().unwrap();
        cleanup();
        let msg = start_daemon().expect("start failed");
        assert!(msg.contains("Daemon started"));
        let status = check_status().expect("status failed");
        assert!(status.contains("running"));
        cleanup();
    }

    #[test]
    fn start_failure() {
        let _lock = TEST_MUTEX.lock().unwrap();
        cleanup();
        env::set_var("OPENASTROVIZD_DAEMON_CMD", "/nonexistent");
        assert!(start_daemon().is_err());
        env::remove_var("OPENASTROVIZD_DAEMON_CMD");
        cleanup();
    }

    #[test]
    fn status_not_running() {
        let _lock = TEST_MUTEX.lock().unwrap();
        cleanup();
        let status = check_status().unwrap();
        assert!(status.contains("not running"));
    }
}
