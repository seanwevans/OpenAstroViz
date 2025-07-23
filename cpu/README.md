# CPU backend

The CPU backend provides a portable reference implementation of the propagation
and conjunction kernels.  It is used by CI so that tests run on any machine and
it allows contributors without an NVIDIA GPU to work on OpenAstroViz.  The
project README notes:

> If you don’t have an NVIDIA GPU, skip `cuda/` and work on the **cpu-simd**
> reference path...

Functionality here will mirror the interfaces defined in `core/` so that either
this backend or the CUDA backend can be selected by `openastrovizd`.

Planned responsibilities

* **SIMD accelerated SGP4** – fast enough for correctness tests and low-end
  deployments.
* **CPU conjunction solver** – analytic miss distance calculation paired with
  the grid pruning kernels in `cuda/`.

Because this crate acts as the reference path, every commit must pass its unit
tests.  See the development philosophy in the
[ROADMAP](../ROADMAP.md#development-philosophy).
