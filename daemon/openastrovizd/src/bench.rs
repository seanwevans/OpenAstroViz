use std::collections::VecDeque;
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
        let work_units = simulate_cuda_cell_grid_conjunctions(24_000)?;
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

#[derive(Clone, Copy, Debug)]
struct Cartesian {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy, Debug)]
struct GridConfig {
    min: f64,
    max: f64,
    voxel_size: f64,
    dim: u32,
}

impl GridConfig {
    fn leo_to_geo() -> Self {
        // ±45,000 km cube comfortably covers LEO through GEO with guard cells.
        let min = -45_000.0;
        let max = 45_000.0;
        let voxel_size = 250.0;
        let dim = ((max - min) / voxel_size) as u32;

        Self {
            min,
            max,
            voxel_size,
            dim,
        }
    }

    fn coords_to_linear_hash(&self, point: Cartesian) -> u32 {
        let nx = self.quantize_axis(point.x);
        let ny = self.quantize_axis(point.y);
        let nz = self.quantize_axis(point.z);

        nx + self.dim * (ny + self.dim * nz)
    }

    fn quantize_axis(&self, axis: f64) -> u32 {
        let clamped = axis.clamp(self.min, self.max - f64::EPSILON);
        let normalized = (clamped - self.min) / self.voxel_size;
        normalized.floor() as u32
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

fn simulate_cuda_cell_grid_conjunctions(object_count: u32) -> Result<u64, BenchError> {
    // Stand-in for CUDA launch overhead.
    thread::sleep(Duration::from_millis(5));

    let grid = GridConfig::leo_to_geo();
    let mut keyed: Vec<(u32, Cartesian)> = (0..object_count)
        .map(|id| {
            // Deterministic pseudo-orbit position generation in km.
            let t = id as f64 * 0.013;
            let point = Cartesian {
                x: (t.sin() * 37_000.0) + ((id % 97) as f64 - 48.0) * 7.5,
                y: (t.cos() * 39_000.0) + ((id % 53) as f64 - 26.0) * 8.0,
                z: ((t * 0.7).sin() * 20_000.0) + ((id % 41) as f64 - 20.0) * 6.0,
            };
            (grid.coords_to_linear_hash(point), point)
        })
        .collect();

    // CUDA equivalent: thrust::sort_by_key / radix sort by voxel hash.
    keyed.sort_unstable_by_key(|(hash, _)| *hash);

    // Build compact voxel ranges over the sorted list.
    let mut voxel_ranges: Vec<(u32, usize, usize)> = Vec::new();
    let mut i = 0;
    while i < keyed.len() {
        let hash = keyed[i].0;
        let start = i;
        while i < keyed.len() && keyed[i].0 == hash {
            i += 1;
        }
        voxel_ranges.push((hash, start, i));
    }

    // Sliding queue keeps only local 3x3x3 neighbors in z-major linear hash space.
    let plane_span = (grid.dim * grid.dim) as u64;
    let mut active: VecDeque<(u32, usize, usize)> = VecDeque::new();
    let mut range_index = 0usize;
    let mut candidate_pairs: u64 = 0;

    while range_index < voxel_ranges.len() {
        let (hash, start, end) = voxel_ranges[range_index];
        let current = hash as u64;

        while let Some((queued_hash, _, _)) = active.front().copied() {
            let queued = queued_hash as u64;
            if current.saturating_sub(queued) > plane_span + grid.dim as u64 + 1 {
                active.pop_front();
            } else {
                break;
            }
        }

        let local_count = (end - start) as u64;
        candidate_pairs =
            candidate_pairs.saturating_add(local_count.saturating_sub(1) * local_count / 2);

        for (neighbor_hash, nstart, nend) in active.iter().copied() {
            if are_neighbor_voxels(hash, neighbor_hash, grid.dim) {
                let left = (end - start) as u64;
                let right = (nend - nstart) as u64;
                candidate_pairs = candidate_pairs.saturating_add(left * right);
            }
        }

        active.push_back((hash, start, end));
        range_index += 1;
    }

    // Lightweight miss-distance stand-in over reduced candidates.
    let work_units = candidate_pairs.saturating_add(keyed.len() as u64);
    black_box(work_units);

    if work_units == 0 {
        return Err(BenchError::Failed);
    }

    Ok(work_units)
}

fn are_neighbor_voxels(a: u32, b: u32, dim: u32) -> bool {
    let (ax, ay, az) = unflatten_hash(a, dim);
    let (bx, by, bz) = unflatten_hash(b, dim);

    (ax as i64 - bx as i64).abs() <= 1
        && (ay as i64 - by as i64).abs() <= 1
        && (az as i64 - bz as i64).abs() <= 1
}

fn unflatten_hash(hash: u32, dim: u32) -> (u32, u32, u32) {
    let plane = dim * dim;
    let z = hash / plane;
    let rem = hash % plane;
    let y = rem / dim;
    let x = rem % dim;
    (x, y, z)
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

    #[test]
    fn voxel_neighbor_predicate_matches_3x3x3_expectation() {
        let dim = GridConfig::leo_to_geo().dim;
        let center = dim + dim * (dim + dim * dim);

        assert!(are_neighbor_voxels(center, center, dim));
        assert!(are_neighbor_voxels(center, center + 1, dim));
        assert!(are_neighbor_voxels(center, center + dim + 1, dim));
        assert!(!are_neighbor_voxels(center, center + 2, dim));
    }

    #[test]
    fn hash_round_trip_is_stable() {
        let grid = GridConfig::leo_to_geo();
        let point = Cartesian {
            x: 42.0,
            y: -1_337.0,
            z: 24_200.0,
        };

        let hash = grid.coords_to_linear_hash(point);
        let (x, y, z) = unflatten_hash(hash, grid.dim);

        assert!(x < grid.dim && y < grid.dim && z < grid.dim);
    }
}
