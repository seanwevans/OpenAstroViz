use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

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

#[derive(Clone)]
struct ServiceState {
    health: Arc<RwLock<SpaceHealthMetrics>>,
}

#[derive(Clone, Debug)]
struct ConjunctionEvent {
    observed_at: SystemTime,
    miss_distance_km: f64,
    relative_velocity_kps: f64,
}

#[derive(Clone)]
struct ConjunctionKernelClient {
    events: Arc<Mutex<VecDeque<ConjunctionEvent>>>,
}

impl ConjunctionKernelClient {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn poll_results(&self, now: SystemTime) -> Vec<ConjunctionEvent> {
        let mut events = self.events.lock().unwrap();
        let elapsed_seed = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let synthesized_event = ConjunctionEvent {
            observed_at: now,
            miss_distance_km: 0.5 + (elapsed_seed % 40) as f64 / 10.0,
            relative_velocity_kps: 7.0 + (elapsed_seed % 60) as f64 / 6.0,
        };
        events.push_back(synthesized_event);

        let retention = Duration::from_secs(26 * 60 * 60);
        while let Some(front) = events.front() {
            if now
                .duration_since(front.observed_at)
                .unwrap_or_default()
                > retention
            {
                events.pop_front();
            } else {
                break;
            }
        }

        events.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SpaceHealthMetrics {
    generated_at: String,
    near_misses_under_1km_24h: usize,
    conjunctions_under_5km_24h: usize,
    critical_conjunctions_last_hour: usize,
    average_relative_velocity_kps_24h: f64,
}

impl Default for SpaceHealthMetrics {
    fn default() -> Self {
        Self {
            generated_at: String::from("1970-01-01T00:00:00Z"),
            near_misses_under_1km_24h: 0,
            conjunctions_under_5km_24h: 0,
            critical_conjunctions_last_hour: 0,
            average_relative_velocity_kps_24h: 0.0,
        }
    }
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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("tokio init failed: {err}")))?;

    runtime.block_on(async move {
        let bind_addr: SocketAddr = env::var("OPENASTROVIZD_HTTP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8000".to_string())
            .parse()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid OPENASTROVIZD_HTTP_ADDR: {err}")))?;

        let poll_secs = env::var("OPENASTROVIZD_HEALTH_POLL_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(15);

        let health = Arc::new(RwLock::new(SpaceHealthMetrics::default()));
        let state = ServiceState { health: Arc::clone(&health) };

        spawn_health_aggregator(Arc::clone(&health), ConjunctionKernelClient::new(), Duration::from_secs(poll_secs));

        let app = Router::new()
            .route("/api/health", get(get_space_health))
            .with_state(state);

        let listener = TcpListener::bind(bind_addr)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::AddrInUse, format!("failed to bind {bind_addr}: {err}")))?;

        axum::serve(listener, app)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("HTTP server error: {err}")))
    })
}

fn spawn_health_aggregator(
    target: Arc<RwLock<SpaceHealthMetrics>>,
    kernel_client: ConjunctionKernelClient,
    cadence: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(cadence);
        loop {
            ticker.tick().await;
            let now = SystemTime::now();
            let events = kernel_client.poll_results(now);
            let metrics = aggregate_health_metrics(&events, now);
            *target.write().await = metrics;
        }
    });
}

fn aggregate_health_metrics(events: &[ConjunctionEvent], now: SystemTime) -> SpaceHealthMetrics {
    let day = Duration::from_secs(24 * 60 * 60);
    let hour = Duration::from_secs(60 * 60);

    let mut near_1km = 0usize;
    let mut under_5km = 0usize;
    let mut critical_hour = 0usize;
    let mut velocities_sum = 0.0f64;
    let mut velocities_count = 0usize;

    for event in events {
        let age = now.duration_since(event.observed_at).unwrap_or_default();
        if age <= day {
            if event.miss_distance_km < 1.0 {
                near_1km += 1;
            }
            if event.miss_distance_km < 5.0 {
                under_5km += 1;
            }
            velocities_sum += event.relative_velocity_kps;
            velocities_count += 1;
        }

        if age <= hour && event.miss_distance_km < 1.0 {
            critical_hour += 1;
        }
    }

    SpaceHealthMetrics {
        generated_at: humantime::format_rfc3339(now).to_string(),
        near_misses_under_1km_24h: near_1km,
        conjunctions_under_5km_24h: under_5km,
        critical_conjunctions_last_hour: critical_hour,
        average_relative_velocity_kps_24h: if velocities_count > 0 {
            velocities_sum / velocities_count as f64
        } else {
            0.0
        },
    }
}

async fn get_space_health(State(state): State<ServiceState>) -> Json<SpaceHealthMetrics> {
    let metrics = state.health.read().await.clone();
    Json(metrics)
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

fn path_from_uri(location: &str) -> PathBuf {
    let normalized = if location.starts_with('/') {
        location.to_string()
    } else {
        format!("/{location}")
    };

    PathBuf::from(normalized)
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
    if let Ok(stat) = fs::read_to_string(stat_path) {
        let state = stat.split_whitespace().nth(2);
        return matches!(state, Some("Z"));
    }
    false
}

#[cfg(not(unix))]
fn process_running(_pid: u32) -> bool {
    false
}

#[cfg(not(unix))]
fn is_zombie(_pid: u32) -> bool {
    false
}

pub fn stop_daemon() -> Result<String, io::Error> {
    let pid_path = pid_file();
    let pid_str = fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid PID file"))?;

    #[cfg(unix)]
    {
        let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    #[cfg(windows)]
    {
        let status_result = take_mock_taskkill_status().unwrap_or_else(|| {
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status()
        });

        let status = status_result?;
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("taskkill exited with status {status}"),
            ));
        }
    }

    fs::remove_file(&pid_path)?;
    Ok(format!("Daemon stopped (pid {pid})"))
}

pub fn check_status() -> Result<String, io::Error> {
    let pid_path = pid_file();
    if !pid_path.exists() {
        return Ok(String::from("Daemon is not running"));
    }

    let pid_str = fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid PID file"))?;

    if process_running(pid) {
        Ok(format!("Daemon is running with pid {pid}"))
    } else {
        let _ = fs::remove_file(pid_path);
        Ok(String::from("Daemon is not running"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_readiness_target_requires_scheme() {
        let err = parse_readiness_target("127.0.0.1:8080").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn parse_readiness_target_rejects_unknown_scheme() {
        let err = parse_readiness_target("udp://127.0.0.1:8080").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn parse_readiness_target_validates_tcp_address() {
        let err = parse_readiness_target("tcp://not-a-real-host::").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn parse_readiness_target_accepts_tcp() {
        let target = parse_readiness_target("tcp://127.0.0.1:8080").unwrap();
        assert!(matches!(target, ReadinessTarget::Tcp(addr) if addr == "127.0.0.1:8080"));
    }

    #[test]
    fn parse_readiness_target_accepts_file_path() {
        let target = parse_readiness_target("file:///tmp/openastrovizd.sock").unwrap();
        assert!(
            matches!(target, ReadinessTarget::Path(path) if path == PathBuf::from("/tmp/openastrovizd.sock"))
        );
    }

    #[test]
    fn parse_readiness_target_rejects_empty_file_path() {
        let err = parse_readiness_target("file://").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn socket_ready_checks_file_targets() {
        let temp_path = env::temp_dir().join(format!("openastrovizd-ready-{}", std::process::id()));
        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        let target = format!("file://{}", temp_path.display());
        assert!(!socket_ready(&target));

        fs::write(&temp_path, "ready").unwrap();
        assert!(socket_ready(&target));

        fs::remove_file(&temp_path).unwrap();
    }

    #[test]
    fn aggregate_health_metrics_counts_rolling_windows() {
        let now = SystemTime::now();
        let events = vec![
            ConjunctionEvent {
                observed_at: now - Duration::from_secs(300),
                miss_distance_km: 0.8,
                relative_velocity_kps: 10.0,
            },
            ConjunctionEvent {
                observed_at: now - Duration::from_secs(2 * 3600),
                miss_distance_km: 2.5,
                relative_velocity_kps: 8.0,
            },
            ConjunctionEvent {
                observed_at: now - Duration::from_secs(25 * 3600),
                miss_distance_km: 0.2,
                relative_velocity_kps: 12.0,
            },
        ];

        let aggregated = aggregate_health_metrics(&events, now);
        assert_eq!(aggregated.near_misses_under_1km_24h, 1);
        assert_eq!(aggregated.conjunctions_under_5km_24h, 2);
        assert_eq!(aggregated.critical_conjunctions_last_hour, 1);
        assert_eq!(aggregated.average_relative_velocity_kps_24h, 9.0);
    }
}
