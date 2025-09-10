use std::fs;
#[cfg(not(unix))]
use std::process::Command;

/// Removes the pid file and terminates the daemon process if it is running.
pub fn cleanup() {
    let pid_path = std::env::temp_dir().join("openastrovizd.pid");
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

