use std::hint::black_box;
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

/// Runs a stub benchmark for the given backend.
///
/// This currently performs a dummy computation to provide an example
/// of how benchmarking might be implemented.
pub fn bench_backend(backend: Backend) -> Result<Duration, BenchError> {
    match backend {
        Backend::Cuda | Backend::Cpu => {
            let start = Instant::now();
            // Dummy workload: simple integer sum
            let mut sum: u64 = 0;
            for i in 0..1_000_000 {
                sum = sum.wrapping_add(i);
            }
            let elapsed = start.elapsed();
            // use sum so optimizer doesn't remove loop
            black_box(sum);
            Ok(elapsed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;

    #[test]
    fn bench_returns_duration() {
        let dur = bench_backend(Backend::Cuda).expect("benchmark failed");
        assert!(dur > Duration::ZERO);

        let dur = bench_backend(Backend::Cpu).expect("benchmark failed");
        assert!(dur > Duration::ZERO);
    }
}
