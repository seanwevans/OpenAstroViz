use openastroviz_core::{GpuBackend, GpuBackendError};
use wasm_bindgen::prelude::*;

const WORKGROUP_SIZE: u32 = 64;

/// WGSL compute kernel scaffold for the FP32 SGP4 pipeline.
///
/// TODO: Port the existing CUDA FP32 orbital update math fully; this shader currently
/// establishes the dispatch and invocation shape so the React client can drive it.
const SGP4_FP32_WGSL: &str = include_str!("../shaders/sgp4_fp32.wgsl");

#[derive(Debug)]
pub struct WebGpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl WebGpuBackend {
    pub async fn initialize() -> Result<Self, GpuBackendError> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok_or_else(|| GpuBackendError::Dispatch("no WebGPU adapter available".to_string()))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("openastroviz-webgpu-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .map_err(|err| GpuBackendError::Dispatch(err.to_string()))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("openastroviz-sgp4-fp32"),
            source: wgpu::ShaderSource::Wgsl(SGP4_FP32_WGSL.into()),
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("openastroviz-sgp4-fp32-pipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        Ok(Self {
            device,
            queue,
            pipeline,
        })
    }
}

impl GpuBackend for WebGpuBackend {
    fn name(&self) -> &'static str {
        "webgpu"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn dispatch_sgp4_fp32_step(&self, batch_size: u32) -> Result<(), GpuBackendError> {
        if batch_size == 0 {
            return Err(GpuBackendError::InvalidBatchSize(batch_size));
        }

        let workgroups = batch_size.div_ceil(WORKGROUP_SIZE);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("openastroviz-sgp4-fp32-encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("openastroviz-sgp4-fp32-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WebGpuBackendHandle {
    backend: WebGpuBackend,
}

#[wasm_bindgen]
impl WebGpuBackendHandle {
    #[wasm_bindgen(js_name = backendName)]
    pub fn backend_name(&self) -> String {
        self.backend.name().to_string()
    }

    #[wasm_bindgen(js_name = dispatchSgp4Fp32Step)]
    pub fn dispatch_sgp4_fp32_step(&self, batch_size: u32) -> Result<(), JsValue> {
        self.backend
            .dispatch_sgp4_fp32_step(batch_size)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }
}

#[wasm_bindgen(js_name = initWebGpuBackend)]
pub async fn init_webgpu_backend() -> Result<WebGpuBackendHandle, JsValue> {
    let backend = WebGpuBackend::initialize()
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    Ok(WebGpuBackendHandle { backend })
}
