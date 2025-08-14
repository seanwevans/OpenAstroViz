use clap::{Parser, Subcommand};

mod backend;
mod bench;
use backend::Backend;
use bench::{bench_backend, BenchError};

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
            match bench_backend(backend) {
                Ok(duration) => {
                    println!("Benchmark for {backend:?} completed in {:?}", duration);
                }
                Err(BenchError::Unsupported) => {
                    eprintln!("Backend {backend:?} is unsupported");
                    std::process::exit(1);
                }
                Err(BenchError::Failed) => {
                    eprintln!("Benchmark for {backend:?} failed");
                    std::process::exit(1);
                }
            }
        }
        None => {
            println!("openastrovizd {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
