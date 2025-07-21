# CUDA style guide

These conventions keep the CUDA codebase consistent and easy to read.

## Formatting

* Format all `.cu` and `.cuh` files with `clang-format -style=file`.
* Use 4-space indentation and wrap lines at 100 characters.
* Place `__host__` and `__device__` qualifiers on the same line as the return type.
* Group kernel launch parameters with spaces:
  `kernel<<<grid, block, shared, stream>>>(args);`

## Best practices

* Minimise host/device memory transfers and reuse allocated buffers.
* Avoid allocating or freeing device memory inside kernels.
* Prefer explicit grid and block sizes over defaulted launches.
* Document assumptions and math in comments above each kernel.

