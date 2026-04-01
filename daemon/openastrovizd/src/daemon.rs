use std::env;
use std::fs;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

fn default_binary_path() -> io::Result<String> {
    if let Ok(path) = env::var("CARGO_BIN_EXE_openastrovizd") {
        return Ok(path);
    }

    let current = env::current_exe()?;
    if let Some(parent) = current.parent() {
        if parent.ends_with("deps") {
            if let Some(bin_dir) = parent.parent() {
                let mut candidate = bin_dir.join(binary_name());
                if cfg!(windows) {
                    candidate.set_extension("exe");
                }
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().into_owned());
                }
            }
        }
    }

    Ok(current.to_string_lossy().into_owned())
}

fn binary_name() -> &'static str {
    "openastrovizd"
}

/// Entry point for the background daemon process. This function blocks
/// indefinitely and is intended to be run by re-invoking the `openastrovizd`
/// binary with the `--run-service` flag.
pub fn run_service(config: Option<&PathBuf>) -> Result<(), io::Error> {
    if let Some(path) = config {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Config file {} was not found", path.display()),
            ));
        }
    }

    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

#[derive(Debug)]
struct DaemonConfig {
    command: String,
    args: Vec<String>,
    readiness_socket: Option<String>,
    readiness_timeout: Duration,
}

#[derive(Debug, Clone)]
enum ReadinessTarget {
    Tcp(String),
    Path(PathBuf),
}

impl DaemonConfig {
    fn from_env() -> io::Result<Self> {
        let command = env::var("OPENASTROVIZD_DAEMON_CMD").or_else(|_| default_binary_path())?;

        let readiness_socket = env::var("OPENASTROVIZD_SOCKET").ok();
        let readiness_timeout = env::var("OPENASTROVIZD_READY_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_secs(5));

        let mut args: Vec<String> = env::var("OPENASTROVIZD_DAEMON_ARGS")
            .map(|value| value.split_whitespace().map(|v| v.to_string()).collect())
            .unwrap_or_else(|_| vec![String::from("--run-service")]);

        if let Some(path) = env::var("OPENASTROVIZD_CONFIG").ok() {
            args.push(String::from("--config"));
            args.push(path);
        }

        Ok(Self {
            command,
            args,
            readiness_socket,
            readiness_timeout,
        })
    }
}

/// Starts the OpenAstroViz daemon by spawning a background process.
///
/// The command used can be overridden with the `OPENASTROVIZD_DAEMON_CMD`
/// environment variable (defaults to the compiled `openastrovizd` binary).
/// Additional arguments can be supplied via `OPENASTROVIZD_DAEMON_ARGS`, and
/// a configuration file path may be provided with `OPENASTROVIZD_CONFIG` (the
/// path is forwarded to the service with a `--config` flag). When spawning the
/// background service, this function waits until either a readiness socket is
/// available (as specified by `OPENASTROVIZD_SOCKET`) or the daemon remains
/// healthy for a short period before writing the PID file. Failures surfaced
/// on stderr or an early exit are propagated back to the caller.
///
/// # Examples
/// ```ignore
/// # Run via CLI (current architecture)
/// $ openastrovizd daemon start
/// Daemon started with pid 12345
///
/// # Equivalent internal call path used by the CLI command handler:
/// # daemon::start_daemon()?;
/// ```
pub fn start_daemon() -> Result<String, io::Error> {
    let pid_path = pid_file();
    match fs::read_to_string(&pid_path) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if process_running(pid) {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!("Daemon already running with pid {pid}"),
                    ));
                }
            }
            let _ = fs::remove_file(&pid_path);
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }

    let config = DaemonConfig::from_env()?;

    let mut child = Command::new(&config.command)
        .args(&config.args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    match wait_for_readiness(&mut child, &config) {
        Ok(()) => {
            forward_child_stderr(&mut child);
            let pid = child.id();
            fs::write(&pid_path, pid.to_string())?;

            Ok(format!("Daemon started with pid {pid}"))
        }
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(e)
        }
    }
}

#[cfg(unix)]
fn kill_result_indicates_running(kill_result: i32, errno: i32) -> bool {
    if kill_result == 0 {
        true
    } else if kill_result == -1 && errno == libc::EPERM {
        true
    } else {
        false
    }
}

#[cfg(unix)]
fn process_running(pid: u32) -> bool {
    unsafe {
        let result = libc::kill(pid as i32, 0);
        let errno = if result == 0 {
            0
        } else {
            *libc::__errno_location()
        };
        if result == -1 && errno == libc::ESRCH {
            return false;
        }
        let running = kill_result_indicates_running(result, errno);
        running && !is_zombie(pid)
    }
}

fn wait_for_readiness(child: &mut Child, config: &DaemonConfig) -> io::Result<()> {
    let readiness_target = config
        .readiness_socket
        .as_deref()
        .map(parse_readiness_target)
        .transpose()?;
    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait()? {
            let stderr = read_child_stderr(child);
            let msg = match status.code() {
                Some(code) => format!("Daemon process exited with status {code}"),
                None => String::from("Daemon process terminated by signal"),
            };

            let detail = stderr
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!(": {s}"))
                .unwrap_or_default();

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{msg}{detail}"),
            ));
        }

        if let Some(target) = &readiness_target {
            if readiness_target_ready(target) {
                return Ok(());
            }
        }

        if config.readiness_socket.is_none() && start.elapsed() >= Duration::from_millis(200) {
            return Ok(());
        }

        if start.elapsed() >= config.readiness_timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "Timed out waiting for daemon readiness after {:?}",
                    config.readiness_timeout
                ),
            ));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn read_child_stderr(child: &mut Child) -> Option<String> {
    let mut stderr = child.stderr.take()?;
    let mut buf = String::new();
    match io::Read::read_to_string(&mut stderr, &mut buf) {
        Ok(_) => Some(buf),
        Err(_) => None,
    }
}

fn forward_child_stderr(child: &mut Child) {
    if let Some(mut stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let _ = io::copy(&mut stderr, &mut io::stderr());
        });
    }
}

#[cfg(test)]
fn socket_ready(target: &str) -> bool {
    parse_readiness_target(target)
        .map(|parsed| readiness_target_ready(&parsed))
        .unwrap_or(false)
}

fn parse_readiness_target(target: &str) -> io::Result<ReadinessTarget> {
    let (scheme, location) = target.split_once("://").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "OPENASTROVIZD_SOCKET must include a scheme: tcp://, unix://, or file://",
        )
    })?;

    match scheme {
        "tcp" => {
            let mut addrs = location.to_socket_addrs().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid tcp socket address `{location}`: {e}"),
                )
            })?;
            if addrs.next().is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid tcp socket address `{location}`"),
                ));
            }
            Ok(ReadinessTarget::Tcp(location.to_string()))
        }
        "file" | "unix" => {
            if location.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{}:// must include a filesystem path", scheme),
                ));
            }
            Ok(ReadinessTarget::Path(path_from_uri(location)))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Unsupported OPENASTROVIZD_SOCKET scheme `{scheme}`; expected tcp://, unix://, or file://"
            ),
        )),
    }
}

#[cfg(windows)]
fn path_from_uri(location: &str) -> PathBuf {
    let bytes = location.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b':' {
        PathBuf::from(&location[1..])
    } else {
        PathBuf::from(location)
    }
}

#[cfg(not(windows))]
fn path_from_uri(location: &str) -> PathBuf {
    PathBuf::from(location)
}

fn readiness_target_ready(target: &ReadinessTarget) -> bool {
    match target {
        ReadinessTarget::Tcp(addr) => TcpStream::connect(addr).is_ok(),
        ReadinessTarget::Path(path) => path.exists(),
    }
}

#[cfg(unix)]
fn is_zombie(pid: u32) -> bool {
    let stat_path = format!("/proc/{pid}/stat");
    if let Ok(contents) = fs::read_to_string(stat_path) {
        if let Some(state) = contents.split_whitespace().nth(2) {
            return state == "Z";
        }
    }
    false
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
/// ```ignore
/// # Start and inspect daemon state through the executable:
/// $ openastrovizd daemon start
/// Daemon started with pid 12345
/// $ openastrovizd daemon status
/// Daemon is running with pid 12345
///
/// # Equivalent internal call path used by `openastrovizd daemon status`:
/// # let status = daemon::check_status()?;
/// # println!("{status}");
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
    let pid_str = match fs::read_to_string(&pid_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(String::from("Daemon is not running"));
        }
        Err(e) => return Err(e),
    };
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

    let wait_timeout = Duration::from_secs(5);
    let deadline = Instant::now() + wait_timeout;

    let mut stopped = false;
    loop {
        if !process_running(pid) {
            stopped = true;
            break;
        }

        if Instant::now() >= deadline {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    if stopped {
        fs::remove_file(pid_path)?;
        Ok(String::from("Daemon stopped"))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Daemon with pid {pid} did not stop within {:?}; pid file left intact",
                wait_timeout
            ),
        ))
    }
}

#[cfg(test)]
#[path = "../tests/util/mod.rs"]
mod util;

#[cfg(test)]
mod tests {
    use super::util;
    use super::*;
    use std::sync::Mutex;

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

    #[test]
    fn stop_without_pid_file_returns_not_running() {
        let _lock = TEST_MUTEX.lock().unwrap();
        util::cleanup();
        let result = stop_daemon();
        assert!(matches!(result, Ok(ref msg) if msg == "Daemon is not running"));
    }

    #[cfg(unix)]
    #[test]
    fn process_running_treats_eperm_as_running() {
        assert!(kill_result_indicates_running(-1, libc::EPERM));
        assert!(!kill_result_indicates_running(-1, libc::ESRCH));
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

    #[test]
    fn start_rejects_running_daemon_and_cleans_stale_pid() {
        let _lock = TEST_MUTEX.lock().unwrap();
        util::cleanup();

        let pid_path = pid_file();

        let first_msg = start_daemon().expect("first start failed");
        assert!(first_msg.contains("Daemon started"));

        let second_attempt = start_daemon();
        assert!(second_attempt.is_err());
        let err = second_attempt.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

        util::cleanup();

        let stale_pid = 999_999u32.to_string();
        fs::write(&pid_path, &stale_pid).expect("should write stale pid file");
        assert!(pid_path.exists(), "pid file should exist before restart");

        let restart_msg = start_daemon().expect("restart should succeed after stale pid");
        assert!(restart_msg.contains("Daemon started"));
        let new_pid_str =
            fs::read_to_string(&pid_path).expect("pid file should exist after restart");
        assert_ne!(stale_pid, new_pid_str.trim());

        util::cleanup();
    }

    #[test]
    fn socket_ready_requires_scheme() {
        assert!(!socket_ready("127.0.0.1:4242"));
        assert!(!socket_ready("/tmp/openastrovizd.sock"));
    }

    #[test]
    fn parse_readiness_target_rejects_invalid_tcp_target() {
        let err = parse_readiness_target("tcp://not a socket").expect_err("must reject bad tcp");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn parse_readiness_target_accepts_file_and_unix_schemes() {
        let file_target =
            parse_readiness_target("file:///tmp/openastrovizd.sock").expect("file uri parses");
        let unix_target =
            parse_readiness_target("unix:///tmp/openastrovizd.sock").expect("unix uri parses");

        assert!(matches!(file_target, ReadinessTarget::Path(_)));
        assert!(matches!(unix_target, ReadinessTarget::Path(_)));
    }
}
