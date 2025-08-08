use clap::ValueEnum;
use std::fmt;

/// Supported backends for benchmarking.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Backend {
    /// CUDA backend
    Cuda,
    /// CPU backend (currently unsupported)
    Cpu,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Backend::Cuda => "cuda",
            Backend::Cpu => "cpu",
        };
        write!(f, "{name}")
    }
}
