# Core module

The **core** crate will own OpenAstroViz's shared orbital mechanics logic. It
will expose data types such as TLE parsing, time systems and coordinate
transforms, and will host the physics models used by every backend.  A key
design decision is that all GPU‑accelerated kernels will live behind a
`GpuBackend` trait so that either a CPU or CUDA implementation can satisfy the
same API.  This principle is called out in the project
[roadmap](../ROADMAP.md) under *CUDA‑first, WebGPU‑ready*.

Both the `cpu/` and `cuda/` folders will depend on this crate.  `core` provides
the algorithms and trait definitions while the backends provide concrete
implementations optimised for their respective targets.

Planned responsibilities include:

* **SGP4 propagator** – reference implementation used by tests and both
  backends (see milestone *M1 – CUDA reference propagator* in the roadmap).
* **Conjunction logic** – cell‑grid pruning and analytic miss‑distance solver
  (roadmap *M2*).
* **Common math utilities** – vector types, frame conversions and helpers used
  by the daemon and web client.

For build instructions and development philosophy refer to the top level
[README](../README.md) and [ROADMAP](../ROADMAP.md).
