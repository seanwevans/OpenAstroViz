# Daemon module

`openastrovizd` is the Rust service that orchestrates the compute backends and
streams data to the web client.  It exposes a small command‑line interface –
`start`, `status` and `bench` – and will serve a WebSocket API for real‑time
orbit information.

During early milestones the daemon mostly shells out to the CPU or CUDA path
for propagation work.

See the [main README](../README.md) for quick start commands and the overall
project layout.
