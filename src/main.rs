use clap::{Parser, Subcommand};

mod api;
mod cache;
mod commands;
mod config;

#[derive(Parser)]
#[command(name = "ytm")]
#[command(about = "YouTube terminal music player")]
struct Cli {
    /// Search term (shortcut for `ytm search <term>`) - supports multiple words
    query: Vec<String>,

    /// Play in background (release terminal, always audio-only)
    #[arg(short, long, global = true)]
    background: bool,

    /// Enable video window (by default, runs audio-only)
    #[arg(short = 'v', long, global = true)]
    video: bool,

    /// Print version and exit
    #[arg(long, global = true)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(short, long)]
        background: bool,
        #[arg(short, long)]
        api: Option<String>,
    },
    Play { url: String, #[arg(short, long)] background: bool },
    Pause,
    Next,
    Prev,
    Stop,
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("ytm version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // By default, audio-only. -v enables video. -b (background) always disables video.
    let no_video = !cli.video || cli.background;

    // Only run search_and_play if a query is provided and no subcommand is present
    if !cli.query.is_empty() && cli.command.is_none() {
        let query = cli.query.join(" ");
        return commands::search_and_play(&query, None, cli.background, no_video).await;
    }

    match cli.command {
        Some(Commands::Search { query, background, api }) => {
            let no_video = !cli.video || background;
            commands::search_and_play(&query, api, background, no_video).await
        }
        Some(Commands::Play { url, background }) => {
            let no_video = !cli.video || background;
            commands::play(&url, background, no_video)
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
