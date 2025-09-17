use clap::{Parser, Subcommand};

mod api;
mod cache;
mod commands;
mod config;
mod mpv;

#[derive(Parser)]
#[command(name = "ytm")]
#[command(about = "YouTube terminal music player")]
#[command(version)]
struct Cli {
    /// Search term (shortcut for `ytm search <term>`) - supports multiple words
    query: Vec<String>,

    /// Enable video window (default is audio-only)
    #[arg(short = 'v', long, help = "Enable video window (default is audio-only)")]
    video: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(short, long)]
        api: Option<String>,
    },
    Play { url: String },
    Pause,
    Next,
    Prev,
    Stop,
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // By default, audio-only. -v enables video.
    let no_video = !cli.video;

    if !cli.query.is_empty() {
        let query = cli.query.join(" ");
        return commands::search_and_play(&query, None, no_video).await;
    }

    match cli.command {
        Some(Commands::Search { query, api }) => {
            commands::search_and_play(&query, api, no_video).await
        }
        Some(Commands::Play { url }) => {
            commands::play(&url, no_video)
        }
        Some(Commands::Pause) => commands::pause(),
        Some(Commands::Next) => commands::next(),
        Some(Commands::Prev) => commands::prev(),
        Some(Commands::Stop) => commands::stop(),
        Some(Commands::Status) => commands::status(),
        None => {
            eprintln!("Usage: ytm <query> or ytm search <query>");
            Ok(())
        }
    }
}
