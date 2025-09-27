use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(all(test, windows))]
static TASKKILL_STATUS: std::sync::Mutex<Option<io::Result<std::process::ExitStatus>>> =
    std::sync::Mutex::new(None);

#[cfg(all(test, windows))]
fn take_mock_taskkill_status() -> Option<io::Result<std::process::ExitStatus>> {
    TASKKILL_STATUS.lock().unwrap().take()
}

#[cfg(all(test, windows))]
pub(crate) fn set_mock_taskkill_status(result: io::Result<std::process::ExitStatus>) {
    *TASKKILL_STATUS.lock().unwrap() = Some(result);
}

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
///
/// # Examples
/// ```no_run
/// use openastrovizd::daemon::start_daemon;
///
/// # fn main() -> Result<(), std::io::Error> {
/// start_daemon()?;
/// # Ok(())
/// # }
/// ```
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
///
/// # Examples
/// ```no_run
/// use openastrovizd::daemon::{start_daemon, check_status};
///
/// # fn main() -> Result<(), std::io::Error> {
/// start_daemon()?;
/// let status = check_status()?;
/// println!("{status}");
/// # Ok(())
/// # }
/// ```
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

/// Stops the OpenAstroViz daemon by reading the PID file, sending a termination
/// signal to the process and removing the PID file.
pub fn stop_daemon() -> Result<String, io::Error> {
    let pid_path = pid_file();
    let pid_str = fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid PID"))?;

    #[cfg(unix)]
    unsafe {
        if libc::kill(pid as i32, libc::SIGTERM) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    #[cfg(not(unix))]
    {
        let status = run_taskkill(pid)?;
        if !status.success() {
            let code = status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| String::from("unknown"));
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("taskkill exited with unsuccessful status code {code}"),
            ));
        }
    }

    fs::remove_file(pid_path)?;
    Ok(String::from("Daemon stopped"))
}

#[cfg(not(unix))]
fn run_taskkill(pid: u32) -> io::Result<std::process::ExitStatus> {
    #[cfg(all(test, windows))]
    {
        if let Some(result) = take_mock_taskkill_status() {
            return result;
        }
    }

    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status()
}

#[cfg(test)]
#[path = "../tests/util/mod.rs"]
mod util;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use super::util;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn start_and_status_success() {
        let _lock = TEST_MUTEX.lock().unwrap();
        util::cleanup();
        let msg = start_daemon().expect("start failed");
        assert!(msg.contains("Daemon started"));
        let status = check_status().expect("status failed");
        assert!(status.contains("running"));
        util::cleanup();
    }

    #[test]
    fn start_failure() {
        let _lock = TEST_MUTEX.lock().unwrap();
        util::cleanup();
        env::set_var("OPENASTROVIZD_DAEMON_CMD", "/nonexistent");
        assert!(start_daemon().is_err());
        env::remove_var("OPENASTROVIZD_DAEMON_CMD");
        util::cleanup();
    }

    #[test]
    fn status_not_running() {
        let _lock = TEST_MUTEX.lock().unwrap();
        util::cleanup();
        let status = check_status().unwrap();
        assert!(status.contains("not running"));
    }

    #[cfg(windows)]
    #[test]
    fn stop_daemon_returns_error_on_taskkill_failure() {
        use std::os::windows::process::ExitStatusExt;

        let _lock = TEST_MUTEX.lock().unwrap();
        let pid_path = pid_file();
        let _ = std::fs::remove_file(&pid_path);
        std::fs::write(&pid_path, "4242").unwrap();

        super::set_mock_taskkill_status(Ok(std::process::ExitStatus::from_raw(1)));

        let err = stop_daemon().expect_err("expected taskkill failure to propagate");
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert!(pid_path.exists());

        let _ = std::fs::remove_file(pid_path);
    }
}
