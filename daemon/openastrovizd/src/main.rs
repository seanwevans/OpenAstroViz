use std::fmt;

use clap::{Parser, Subcommand, ValueEnum};

mod bench;
use bench::bench_backend;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Backend {
    Cuda,
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

#[derive(Parser)]
#[command(author, version, about = "OpenAstroViz daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Start,
    /// Show daemon status
    Status,
    /// Run benchmarks for a backend
    Bench {
        /// Backend to benchmark (e.g. cuda)
        backend: Backend,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start) => {
            println!("Starting daemon... (placeholder)");
        }
        Some(Commands::Status) => {
            println!("Daemon status: unknown (placeholder)");
        }
        Some(Commands::Bench { backend }) => {
            bench_backend(backend);
        }
        None => {
            println!("openastrovizd {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
