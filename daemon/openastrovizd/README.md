# openastrovizd

`openastrovizd` is the command line daemon that powers OpenAstroViz. It exposes the compute backends (CUDA or CPU) to the web client and other tools. The daemon lives in the `daemon/openastrovizd` crate and is built as part of the workspace.

## Building

Run the standard cargo build command from the repository root:

```bash
cargo build -p openastrovizd
```

This compiles the daemon in debug mode. Use `--release` for an optimized binary.

## Commands

The daemon provides a few subcommands:

- `start` – launch the background service
- `status` – query whether the service is running
- `stop` – terminate the running daemon and clean up the PID file
- `bench <backend>` – run performance benchmarks for a backend (e.g. `cuda`)

Running `openastrovizd` with no arguments prints the version.

When the `start` subcommand is executed the daemon spawns a background
process and writes its process ID to a file named `openastrovizd.pid` in the
system temporary directory. The `status` subcommand reads this file and checks
whether the recorded process is still alive, allowing the daemon to be
monitored with simple status queries. The `stop` subcommand terminates the
running daemon and removes the PID file to prevent stale state.

## Example usage

```bash
$ cargo run -p openastrovizd -- start      # start the daemon
$ cargo run -p openastrovizd -- status     # check if it's alive
$ cargo run -p openastrovizd -- stop       # terminate the daemon and remove the PID file
$ cargo run -p openastrovizd -- bench cuda # benchmark the CUDA backend
```

The daemon is the link between the high‑level web interface and the low‑level compute kernels, serving orbit propagation results over local APIs.


## Live orbital catalog refresh

When the daemon runs in service mode (`start` command), it now launches an
asynchronous refresh task backed by Tokio. The task downloads the public
CelesTrak active-satellite TLE catalog once every 24 hours using `reqwest`,
parses each 3-line record, builds Vallado-compatible propagators, and computes
an epoch state vector per object. The in-memory catalog is protected by a
`tokio::sync::RwLock` so future WebSocket handlers can read the current orbital
state while refreshes hot-swap the catalog without disconnecting clients.

## Startup environment variables

`openastrovizd start` supports these environment variables:

- `OPENASTROVIZD_DAEMON_CMD`: override daemon executable path.
- `OPENASTROVIZD_DAEMON_ARGS`: override daemon arguments.
- `OPENASTROVIZD_CONFIG`: config file passed as `--config <path>`.
- `OPENASTROVIZD_READY_TIMEOUT_MS`: readiness timeout (milliseconds).
- `OPENASTROVIZD_SOCKET`: readiness target URI with an explicit scheme:
  - `tcp://host:port`
  - `unix:///path/to/socket`
  - `file:///path/to/ready/file`

Examples:

```bash
OPENASTROVIZD_SOCKET=tcp://127.0.0.1:8765
OPENASTROVIZD_SOCKET=unix:///tmp/openastrovizd.sock
OPENASTROVIZD_SOCKET=file:///tmp/openastrovizd.ready
OPENASTROVIZD_SOCKET=file:///C:/Temp/openastrovizd.ready  # Windows path
OPENASTROVIZD_READY_TIMEOUT_MS=8000
```
