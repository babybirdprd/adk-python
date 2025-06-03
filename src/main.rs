//! ADK CLI binary

use clap::{Parser, Subcommand};
use google_adk::cli::commands::{ApiServerCommand, CreateCommand, EvalCommand, RunCommand, WebCommand};
use google_adk::{init, Result};
use std::process;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "adk")]
#[command(about = "Agent Development Kit CLI tools")]
#[command(version = google_adk::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new agent project
    Create(CreateCommand),
    /// Run an interactive CLI for an agent
    Run(RunCommand),
    /// Evaluate an agent
    Eval(EvalCommand),
    /// Start a web server with UI for agents
    Web(WebCommand),
    /// Start a FastAPI server for agents
    #[command(name = "api_server")]
    ApiServer(ApiServerCommand),
}

#[tokio::main]
async fn main() {
    // Initialize the library
    if let Err(e) = init() {
        eprintln!("Failed to initialize ADK: {}", e);
        process::exit(1);
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create(cmd) => cmd.execute().await,
        Commands::Run(cmd) => cmd.execute().await,
        Commands::Eval(cmd) => cmd.execute().await,
        Commands::Web(cmd) => cmd.execute().await,
        Commands::ApiServer(cmd) => cmd.execute().await,
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        process::exit(1);
    }

    info!("Command completed successfully");
}
