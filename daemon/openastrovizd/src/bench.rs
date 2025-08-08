use std::time::{Duration, Instant};

use crate::backend::Backend;

/// Errors that can occur while benchmarking.
#[derive(Debug)]
pub enum BenchError {
    /// The requested backend is not yet supported.
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
        Backend::Cuda => {
            let start = Instant::now();
            // Dummy workload: simple integer sum
            let mut sum: u64 = 0;
            for i in 0..1_000_000 {
                sum = sum.wrapping_add(i);
            }
            let elapsed = start.elapsed();
            // use sum so optimizer doesn't remove loop
            let _ = sum;
            Ok(elapsed)
        }
        Backend::Cpu => Err(BenchError::Unsupported),
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
    }

    #[test]
    fn bench_returns_error_for_unsupported() {
        assert!(matches!(
            bench_backend(Backend::Cpu),
            Err(BenchError::Unsupported)
        ));
    }
}
