use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

use crate::backend::Backend;

/// Errors that can occur while benchmarking.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BenchError {
    /// The requested backend is not supported.
    ///
    /// Currently supported backends are [`Backend::Cpu`] and [`Backend::Cuda`].
    Unsupported,
    /// The benchmark failed to execute.
    Failed,
}

/// Captures the outcome of a backend benchmark run.
#[derive(Debug, PartialEq)]
pub struct BenchOutcome {
    pub duration: Duration,
    pub work_units: u64,
}

trait BenchRunner {
    fn run(&self) -> Result<BenchOutcome, BenchError>;
}

struct CpuRunner;

impl BenchRunner for CpuRunner {
    fn run(&self) -> Result<BenchOutcome, BenchError> {
        let start = Instant::now();
        let work_units = simulate_cpu_propagation(50_000)?;
        Ok(BenchOutcome {
            duration: start.elapsed(),
            work_units,
        })
    }
}

struct CudaRunner;

impl BenchRunner for CudaRunner {
    fn run(&self) -> Result<BenchOutcome, BenchError> {
        let start = Instant::now();
        let work_units = simulate_cuda_propagation(10_000)?;
        Ok(BenchOutcome {
            duration: start.elapsed(),
            work_units,
        })
    }
}

/// Runs a stub benchmark for the given backend.
///
/// This currently performs a dummy computation to provide an example
/// of how benchmarking might be implemented.
pub fn bench_backend(backend: Backend) -> Result<BenchOutcome, BenchError> {
    match backend {
        Backend::Cuda => CudaRunner.run(),
        Backend::Cpu => CpuRunner.run(),
    }
}

fn simulate_cpu_propagation(steps: u64) -> Result<u64, BenchError> {
    // Stand-in for CPU propagator: iteratively update a pseudo state vector.
    let mut state = [1.0_f64, 0.5, -0.25];
    let mut total = 0.0_f64;

    for tick in 0..steps {
        state[0] += state[1] * 0.001;
        state[1] += state[2] * 0.001;
        state[2] = -(state[0] * 0.0001).sin();
        total += state[0].abs() + state[1].abs() + state[2].abs() + tick as f64 * 1e-6;
    }

    let work_units = (total * 10_000.0) as u64;
    black_box(&state);
    if work_units == 0 {
        return Err(BenchError::Failed);
    }

    Ok(work_units)
}

fn simulate_cuda_propagation(batch_size: u64) -> Result<u64, BenchError> {
    // Stand-in for CUDA propagator: emulate kernel setup overhead and batched computation.
    let pre_launch_overhead = Duration::from_millis(5);
    thread::sleep(pre_launch_overhead);

    let mut kernel_accumulator: u64 = 0;
    for batch in 0..batch_size {
        // Simulate a fused multiply-add heavy kernel over multiple lanes.
        let lane = (batch % 256) as f64;
        let signal = (lane.cos() + lane.sin()).abs();
        let energy = signal * (1.0 + (batch as f64 * 1e-4));
        kernel_accumulator = kernel_accumulator.wrapping_add((energy * 1_000.0) as u64 + batch);
    }

    black_box(kernel_accumulator);

    if kernel_accumulator == 0 {
        return Err(BenchError::Failed);
    }

    Ok(kernel_accumulator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;

    #[test]
    fn bench_returns_non_zero_work() {
        let cuda = bench_backend(Backend::Cuda).expect("benchmark failed");
        assert!(cuda.duration > Duration::ZERO);
        assert!(cuda.work_units > 0);

        let cpu = bench_backend(Backend::Cpu).expect("benchmark failed");
        assert!(cpu.duration > Duration::ZERO);
        assert!(cpu.work_units > 0);
    }

    #[test]
    fn cuda_and_cpu_paths_are_distinct() {
        let cuda = bench_backend(Backend::Cuda).expect("benchmark failed");
        let cpu = bench_backend(Backend::Cpu).expect("benchmark failed");

        assert!(cuda.duration > cpu.duration);
        assert_ne!(cuda.work_units, cpu.work_units);
    }
}
