# openastroviz-webgpu-compute

WebGPU compute scaffolding for OpenAstroViz. This crate introduces a browser-native
`GpuBackend` implementation with a WGSL kernel entrypoint intended for the FP32
SGP4 port from CUDA.

## Build to WebAssembly with wasm-pack

```bash
wasm-pack build webgpu-compute --target web --release --out-dir pkg
```

The generated `pkg/` directory can be consumed directly by the React/Vite frontend,
which can call `initWebGpuBackend()` and then dispatch compute work using
`dispatchSgp4Fp32Step(batchSize)`.
