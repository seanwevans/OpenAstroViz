use clap::{Parser, Subcommand};

mod bench;
use bench::bench_backend;

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
        backend: String,
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
            bench_backend(&backend);
        }
        None => {
            println!("openastrovizd {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
