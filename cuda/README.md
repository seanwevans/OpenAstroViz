# CUDA module

This directory contains the NVIDIA GPU implementation of OpenAstroViz.  All
high‑performance kernels such as SGP4 propagation and conjunction searches will
live here.  The code is written in CUDA C++ and follows the style guide in
[`docs/cuda_style.md`](../docs/cuda_style.md).

These kernels are launched by the `openastrovizd` daemon to accelerate orbit
calculations.

For build instructions and the overall project vision, consult the
[main README](../README.md).
