# OpenAstroViz – Roadmap

> **Versioning:** CalVer `YY.MM.minor`.  CUDA first, WebGPU later.

| Milestone                          | Target CalVer | Theme        | Key deliverables                                                                          |
| ---------------------------------- | ------------- | ------------ | ----------------------------------------------------------------------------------------- |
| **M0 – Bootstrap**                 | 25.07         | Scaffolding  | Repo skeleton, CI, `openastrovizd` daemon skeleton, CUDA toolchain.                       |
| **M1 – CUDA reference propagator** | 25.08         | Correctness  | CUDA SGP4 (FP32 & FP64) + unit tests vs Vallado vectors; WebSocket JSON stream to client. |
| **M2 – Conjunction kernel**        | 25.09         | Safety       | CUDA cell‑grid pruning + CPU analytic solver; emit top‑N close approaches per frame.      |
| **M3 – Web client MVP**            | 25.10         | Visuals      | React/Three.js globe, timeline scrubber, tooltip, close‑approach panel.                   |
| **M4 – Performance hardening**     | 25.11         | Scale        | 100 k objects @ 60 fps on RTX 3060, latency budget < 16 ms frame.                         |
| **M5 – WebGPU PoC**                | 25.12         | Portability  | WASM + WebGPU FP32 SGP4 parity with CUDA FP32; feature‑flag compile.                      |
| **M6 – WebGPU GA**                 | 26.02         | Browser‑only | Safari/Firefox stable support; remove need for local daemon.                              |
| **M7 – Metrics & Story mode**      | 26.03         | Insight      | Dashboard + guided tours API.                                                             |

---

## Development philosophy

1. **CUDA‑first, WebGPU‑ready.** All kernels behind `GpuBackend` trait.
2. **Public by default.** Technical discussion in issues/PRs.
3. **Test every commit.** CPU reference path runs in CI.
4. **Education matters.** We merge tutorial PRs as eagerly as code PRs.

---

## How to contribute

* NVIDIA user? Hack on `cuda/` kernels.
* No GPU? Help with CPU reference, documentation, or web UI.
* Check the board, comment `/claim`, open draft PR early.

Happy hacking – meet you in orbit! 🛰️
