mod api;
mod config;
mod commands;

use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[command(name = "ytm", about = "Search and play YouTube via mpv")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search YouTube for videos
    Search {
        #[arg(num_args(1..), trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Play a video by its index from the last search
    Play {
        index: usize,
    },
    /// Set or replace API key
    Api {
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => commands::search(query).await?,
        Commands::Play { index } => commands::play(index).await?,
        Commands::Api { key } => commands::set_api_key(key).await?,
    }

    Ok(())
}


