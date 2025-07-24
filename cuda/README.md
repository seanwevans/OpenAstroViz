# CUDA backend

The CUDA backend houses the high‑throughput GPU kernels used for real‑time orbit
propagation and conjunction detection.  It is the focus of the early roadmap
milestones – see *M1 – CUDA reference propagator* and *M2 – Conjunction kernel*
in [ROADMAP](../ROADMAP.md).

Kernels implemented here conform to the `GpuBackend` trait defined in `core/` so
that the daemon can switch between CPU and GPU at runtime.  Developers are
encouraged to follow the style guidance in
[docs/cuda_style.md](../docs/cuda_style.md).

Planned responsibilities

* **SGP4 GPU kernels** – FP32/FP64 propagation validated against Vallado test
  vectors.
* **Cell‑grid pruning** – GPU‑accelerated close approach search producing a list
  for the CPU solver in `cpu/`.

Building the backend requires a working CUDA toolchain.  The top level
[README](../README.md#-quick-start-development) shows how to build and benchmark
the CUDA path via the `openastrovizd` bench command.
