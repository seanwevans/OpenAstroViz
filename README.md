# OpenAstroViz

**FlightRadar24 for space – but fully open‑source and physics‑grade accurate.**
OpenAstroViz streams, propagates, and visualises **every tracked object in Earth orbit** – satellites, spent stages, debris – in real time.

> **Mission statement:** Democratise space‑situational awareness.  Provide a transparent, vendor‑neutral SSA stack that anyone can run locally or fork for research.

![OpenAstroViz hero screenshot](docs/assets/hero_placeholder.png)

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

# 2 – build CUDA backend (requires NVIDIA + nvcc)
$ cargo run -p openastrovizd -- bench cuda   # benchmarks, STK vector tests

# 3 – run the web client (connects to local daemon)
$ yarn --cwd web install && yarn --cwd web dev   # http://localhost:5173
```

If you don’t have an NVIDIA GPU, skip `cuda/` and work on the **cpu‑simd** reference path or UI issues labelled *good first issue*.

---

## 🧩 Contributing

Standard [fork → branch → PR](CONTRIBUTING.md) flow.  clang‑format / rustfmt / eslint are enforced via pre‑commit.  CUDA code follows the style guide in `docs/cuda_style.md`.

---

## 📜 License

* **Code:** [MIT](LICENSE) © 2025 OpenAstroViz Contributors
* **Data:** CC0 (derived from U.S. Government public‑domain TLEs)

---

