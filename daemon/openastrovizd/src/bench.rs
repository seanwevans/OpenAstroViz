use std::time::{Duration, Instant};

/// Runs a stub benchmark for the given backend.
///
/// This currently performs a dummy computation to provide
/// an example of how benchmarking might be implemented.
pub fn bench_backend(backend: &str) -> Duration {
    let start = Instant::now();
    // Dummy workload: simple integer sum
    let mut sum: u64 = 0;
    for i in 0..1_000_000 {
        sum = sum.wrapping_add(i);
    }
    let elapsed = start.elapsed();
    println!(
        "Benchmark for backend {backend}: computed sum={} in {:?}",
        sum, elapsed
    );
    elapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_returns_duration() {
        let dur = bench_backend("test");
        assert!(dur > Duration::ZERO);
    }
}
