# Core module

The `core` crate will hold the shared data structures and physics routines used by
all other parts of OpenAstroViz.  Both the CPU reference path and the CUDA kernels
will depend on this code so that they stay in sync.  Keeping the orbital mechanics
logic here also makes it easier to port future backends such as WebGPU.

For a project overview and build instructions see the
[main README](../README.md).
