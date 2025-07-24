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
- `bench <backend>` – run performance benchmarks for a backend (e.g. `cuda`)

Running `openastrovizd` with no arguments prints the version.

## Example usage

```bash
$ cargo run -p openastrovizd -- start      # start the daemon
$ cargo run -p openastrovizd -- status     # check if it's alive
$ cargo run -p openastrovizd -- bench cuda # benchmark the CUDA backend
```

The daemon is the link between the high‑level web interface and the low‑level compute kernels, serving orbit propagation results over local APIs.
