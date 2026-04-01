# OpenAstroViz Daemon

`openastrovizd` is the long-running service that exposes OpenAstroViz compute backends to clients. It runs locally and bridges the web interface with CPU or GPU simulation kernels.

## Basic usage

```
openastrovizd start         # launch the daemon
openastrovizd status        # check if it is running
openastrovizd bench <backend>  # benchmark a backend (e.g. cuda)
```

Starting the daemon spawns a lightweight background process and writes its
process ID to a file in the system temporary directory. Subsequent `status`
checks read this file and verify that the process is still alive, providing a
simple way to monitor the daemon.

## Environment variables for readiness

When launching with readiness checks, `OPENASTROVIZD_SOCKET` must use one of
the following URI formats:

- `tcp://host:port`
- `unix:///path/to/socket`
- `file:///path/to/ready/file`

Examples:

```bash
OPENASTROVIZD_SOCKET=tcp://localhost:8765
OPENASTROVIZD_SOCKET=unix:///tmp/openastrovizd.sock
OPENASTROVIZD_SOCKET=file:///tmp/openastrovizd.ready
OPENASTROVIZD_SOCKET=file:///C:/Temp/openastrovizd.ready  # Windows
```

For detailed instructions and advanced options, see the [openastrovizd crate README](openastrovizd/README.md) or the project documentation.
