# AGENTS.md - YouTube Music CLI (ytm)

## Build/Test Commands
- `cargo build --release` - Build release version
- `cargo run -- <query>` - Run with search query (e.g., `cargo run -- portishead third`)
- `nix develop` - Enter Nix development shell
- `nix build .` - Build with Nix (reproducible)
- No unit tests found in codebase - manual testing via running binary

## Architecture
- **Main modules**: `main.rs` (CLI), `api.rs` (YouTube API), `commands.rs` (mpv integration), `config.rs` (API key storage), `cache.rs` (local caching)
- **YouTube API integration**: Uses YouTube Data API v3 for search, handles videos and playlists
- **Media playback**: Integrates with external `mpv` player via Unix socket communication (`/tmp/ytm-mpv.sock`)
- **Interactive selection**: Uses `fzf` for search result selection
- **Local caching**: Caches API responses to minimize Google API quota usage (100 searches/day limit)
- **Config storage**: API keys stored in `~/.config/ytm/config.json`

## Code Style & Conventions
- **Language**: Rust 2021 edition
- **Error handling**: Uses `anyhow::Result` throughout for error propagation
- **Async**: Tokio runtime for async operations (YouTube API calls)
- **Dependencies**: `clap` for CLI, `reqwest` for HTTP, `serde_json` for JSON, `tokio` for async
- **Imports**: Standard library first, then external crates, then local modules
- **Naming**: Snake_case for functions/variables, PascalCase for types/structs
- **Structs**: Use `#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]` for data types
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `PID_FILE`, `SOCK_PATH`)
- **Module structure**: Single-responsibility modules with clear separation of concerns
