# OpenAstroViz Web Client

This package contains the WebGL front end for the OpenAstroViz stack. It renders a GPU-accelerated 3‑D globe, streams orbital telemetry from the local daemon, and provides SSA tooling such as a timeline scrubber, conjunction highlights, and space-health metrics.

## Features

- **WebGL globe** powered by Three.js with instanced rendering for thousands of spacecraft and debris objects.
- **Timeline scrubber** to scrub forward/backward through propagated states while maintaining a live mode.
- **Real-time telemetry** via WebSocket with automatic HTTP snapshot fallback and in-browser propagation between frames.
- **Conjunction awareness** surfaces close approaches within the active time window and highlights objects on the globe.
- **Space-health dashboard** summarises nominal, warning, critical assets, and debris counts.

## Getting started

```bash
cd web
corepack enable # optional, if Yarn is not configured
yarn install
yarn dev
```

The development server launches at [http://localhost:5173](http://localhost:5173). By default the client expects the OpenAstroViz daemon at `http://localhost:8000`; override via the following environment variables when running `yarn dev`:

```bash
VITE_DAEMON_HTTP="https://example.com" \
VITE_DAEMON_WS="wss://example.com/ws/orbits" \
yarn dev
```

## Build & lint

```bash
yarn build
yarn lint
```

`yarn build` runs TypeScript type-checking followed by a production Vite bundle.
