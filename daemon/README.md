# OpenAstroViz Daemon

`openastrovizd` is the long-running service that exposes OpenAstroViz compute backends to clients. It runs locally and bridges the web interface with CPU or GPU simulation kernels.

## Basic usage

```
openastrovizd start         # launch the daemon
openastrovizd status        # check if it is running
openastrovizd bench <backend>  # benchmark a backend (e.g. cuda)
```

For detailed instructions and advanced options, see the [openastrovizd crate README](openastrovizd/README.md) or the project documentation.
