use clap::{Parser, Subcommand};

mod backend;
mod bench;
mod daemon;
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
    /// Stop the daemon
    Stop,
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
        Some(Commands::Start) => match daemon::start_daemon() {
            Ok(message) => println!("{message}"),
            Err(e) => {
                eprintln!("Failed to start daemon: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::Stop) => match daemon::stop_daemon() {
            Ok(message) => println!("{message}"),
            Err(e) => {
                eprintln!("Failed to stop daemon: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::Status) => match daemon::check_status() {
            Ok(status) => println!("{status}"),
            Err(e) => {
                eprintln!("Failed to check daemon status: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::Bench { backend }) => match bench_backend(backend) {
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
        },
        None => {
            println!("openastrovizd {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
