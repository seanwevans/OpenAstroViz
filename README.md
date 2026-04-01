# OpenAstroViz
<img width="256" alt="LASERS!" src="https://github.com/user-attachments/assets/e6491b05-be89-4c3f-a6ac-fc103126ffc8" />

**FlightRadar24 for space – but fully open‑source and physics‑grade accurate.**
OpenAstroViz streams, propagates, and visualises **every tracked object in Earth orbit** – satellites, spent stages, debris – in real time.

> **Mission statement:** Democratise space‑situational awareness.  Provide a transparent, vendor‑neutral SSA stack that anyone can run locally or fork for research.


---

## ✨  Key features (v0 vision)

* **Live orbital map** – WebGL globe + timeline scrubber (Sputnik → +1 year forecast).
* **CUDA compute backend (v0)** – GPU‑accelerated SGP4 & conjunction kernels (RTX/GeForce/Quadro).
  *We start with NVIDIA CUDA for maximum throughput and mature tooling.*
* **WebGPU compute backend (v2)** – Pure‑browser, cross‑vendor path that eliminates native daemons.  Targeted once Safari/WebGPU FP32 stabilises.
* **Real‑time conjunction alerts** – cell‑grid pruning + analytic miss distance solver.
* **Space‑health dashboard** – daily / weekly / yearly “Top‑5” congestion indicators.
* **Story mode** – replay Iridium‑Cosmos, Starlink shell deployment, etc.

---

## 🚀 Quick start (development)

```bash
# 1 – clone & bootstrap
$ git clone https://github.com/openastroviz/openastroviz.git && cd openastroviz
$ ./scripts/setup.sh          # installs Rust, CUDA, Node.js + pre‑commit hooks
$ rustup override set stable  # ensure the stable toolchain locally

# 2 – build CUDA backend (requires NVIDIA + nvcc)
$ cargo run -p openastrovizd -- bench cuda   # benchmarks, STK vector tests

# 2b – run reference Vallado SGP4 sample (CPU)
$ cargo run -p openastroviz-core --example propagate

# 3 – run the web client (connects to local daemon)
$ yarn --cwd web install && yarn --cwd web dev   # http://localhost:5173
```

If you don’t have an NVIDIA GPU, skip `cuda/` and work on the **cpu‑simd** reference path or UI issues labelled *good first issue*.

The validated Vallado SGP4 implementation now lives in the [`core/`](core) crate and is exercised by regression tests against published state vectors.  The earlier Keplerian proof‑of‑concept has been quarantined to [`docs/archive/poc_sgp.cpp`](docs/archive/poc_sgp.cpp) for historical reference.

### Daemon readiness socket format

When using daemon startup readiness checks, `OPENASTROVIZD_SOCKET` now requires
an explicit scheme:

* `tcp://<host>:<port>` for TCP socket readiness checks.
* `unix:///absolute/path/to/socket` for Unix domain socket/file readiness checks.
* `file:///absolute/path/to/socket-or-marker` for filesystem path readiness checks.

Examples:

```bash
OPENASTROVIZD_SOCKET=tcp://127.0.0.1:8765
OPENASTROVIZD_SOCKET=unix:///tmp/openastrovizd.sock
OPENASTROVIZD_SOCKET=file:///tmp/openastrovizd.ready
OPENASTROVIZD_SOCKET=file:///C:/Temp/openastrovizd.ready  # Windows
```

---

## 🧩 Contributing

Standard [fork → branch → PR](CONTRIBUTING.md) flow.  clang‑format / rustfmt / eslint are enforced via pre‑commit.  CUDA code follows the style guide in `docs/cuda_style.md`.

---

## 📜 License

* **Code:** [MIT](LICENSE) © 2025 OpenAstroViz Contributors
* **Data:** CC0 (derived from U.S. Government public‑domain TLEs)

---
