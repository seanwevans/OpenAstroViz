# CPU module

The `cpu` folder provides the baseline host implementation of the propagator.
Initially it will be single threaded and prioritise correctness.  The GPU
kernels are validated against this path and continuous integration runs its
tests.

Later milestones will explore SIMD and multithreaded optimisations here.

See the [main README](../README.md) for how this module fits into the overall
project.
