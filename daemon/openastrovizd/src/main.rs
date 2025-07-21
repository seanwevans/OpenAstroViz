use clap::{Parser, Subcommand};

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
        None => {
            println!("openastrovizd {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
