use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::api::YouTubeClient;
use crate::cache::Cache;
use crate::config::Config;
use crate::mpv::{self, Mpv};

/// Get the directory for application cache
fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ytm")
}

/// Format seconds as MM:SS string
fn format_time(seconds: Option<f64>) -> String {
    match seconds {
        Some(secs) => {
            let minutes = (secs / 60.0).floor() as u32;
            let seconds = (secs % 60.0).round() as u32;
            format!("{:02}:{:02}", minutes, seconds)
        }
        None => "--:--".to_string(),
    }
}

/// Simple status display 
fn show_detailed_status() -> anyhow::Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    if let Ok(mut mpv_client) = Mpv::connect() {
        if let Ok(status) = mpv_client.get_status() {
            if let Some(title) = status.title {
                println!("{}", title);
                
                // Show album or playlist title
                if let Some(album) = &status.album {
                    println!("Album: {}", album);
                } else if let Some(playlist) = &status.playlist_title {
                    println!("Playlist: {}", playlist);
                }
                
                let pos_str = format_time(status.position);
                let dur_str = format_time(status.duration);
                let percentage = if let (Some(pos), Some(dur)) = (status.position, status.duration) {
                    if dur > 0.0 { (pos / dur * 100.0) as u32 } else { 0 }
                } else { 0 };
                
                // Format playlist info
                let playlist_info = if let (Some(pos), Some(count)) = (status.playlist_pos, status.playlist_count) {
                    if count > 1 {
                        format!("[playlist] #{}/{}", pos, count)
                    } else {
                        "[playing]".to_string()
                    }
                } else {
                    "[playing]".to_string()
                };
                
                println!("{}   {} / {} ({}%)", playlist_info, pos_str, dur_str, percentage);
                println!("volume: 100%");
                return Ok(());
            }
        }
    }
    
    println!("Player not responding");
    Ok(())
}

fn show_status_once() -> anyhow::Result<bool> {
    let mut mpv_client = match Mpv::connect() {
        Ok(client) => client,
        Err(_) => {
            println!("(player not running)");
            return Ok(false);
        }
    };

    let status = mpv_client.get_status()?;
    
    if let Some(title) = status.title {
        let pos_str = format_time(status.position);
        let dur_str = format_time(status.duration);
        print!("\r▶ {}  [{} / {}]   ", title, pos_str, dur_str);
        std::io::stdout().flush().ok();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Interactive API key prompt
async fn prompt_for_api_key() -> anyhow::Result<String> {
    use crate::api;
    
    loop {
        print!("Enter your YouTube API key: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let key = input.trim().to_string();
        if api::validate_key(&key).await {
            return Ok(key);
        } else {
            eprintln!("[!] Invalid key, try again.");
        }
    }
}

/// Search YouTube, pick first result, and play
pub async fn search_and_play(query: &str, api: Option<String>, no_video: bool, background: bool) -> anyhow::Result<()> {
    let mut cfg = Config::load()?;
    if let Some(api_key) = api {
        cfg.set_api_key(api_key);
        cfg.save()?;
    }
    
    let key = if cfg.is_api_key_valid().await {
        cfg.api_key().unwrap().clone()
    } else {
        // Prompt for API key interactively
        let new_key = prompt_for_api_key().await?;
        cfg.set_api_key(new_key.clone());
        cfg.save()?;
        new_key
    };

    let cache = Cache::new(cache_dir(), std::time::Duration::from_secs(3600))?;
    let client = YouTubeClient::new(key, cache);

    // Fetch 50 results
    let results = client.search(query, Some(50)).await?;
    if results.is_empty() {
        return Err(anyhow::anyhow!("No results for '{}'", query));
    }

    // Format for fzf: concise info, add [playlist] if needed, hide ID from display
    let mut lines = Vec::new();
    let mut ids = Vec::new();
    let mut is_playlist_vec = Vec::new();
    for item in &results {
        let title = &item.snippet.title;
        let channel = &item.snippet.channel_title;
        let (id_str, label, is_playlist) = if let Some(playlist_id) = &item.id.playlist_id {
            (playlist_id.as_str(), " [playlist]", true)
        } else if let Some(video_id) = &item.id.video_id {
            (video_id.as_str(), "", false)
        } else {
            continue;
        };
        lines.push(format!("{} | {}{}", title, channel, label));
        ids.push(id_str.to_string());
        is_playlist_vec.push(is_playlist);
    }
    let fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = fzf.stdin.as_ref().unwrap();
        for line in &lines {
            writeln!(stdin, "{}", line)?;
        }
    }
    let output = fzf.wait_with_output()?;
    if !output.status.success() {
        return Ok(());
    }
    let selected = String::from_utf8_lossy(&output.stdout);
    if selected.trim().is_empty() {
        return Ok(());
    }
    // Find the index of the selected line to get the corresponding ID
    let selected_line = selected.trim();
    let idx = lines.iter().position(|l| l == selected_line).ok_or_else(|| anyhow::anyhow!("Selection not found"))?;
    let id = &ids[idx];
    let is_playlist = is_playlist_vec[idx];
    let url = if is_playlist {
        format!("https://www.youtube.com/playlist?list={}", id)
    } else {
        format!("https://www.youtube.com/watch?v={}", id)
    };
    play(&url, no_video, background)
}

/// Start mpv either foreground or background with IPC enabled
pub fn play(url: &str, no_video: bool, background: bool) -> anyhow::Result<()> {
    let mut args = Vec::new();
    if no_video {
        args.push("--no-video");
    }
    if background {
        args.push("--input-ipc-server=/tmp/ytm-mpv.sock");
    }
    args.push(url);
    
    if background {
        // Start mpv in background with output suppressed from the start
        let _child = Command::new("mpv")
            .args(&args)
            .stdout(std::process::Stdio::null())  // Suppress stdout immediately
            .stderr(std::process::Stdio::null())  // Suppress stderr immediately  
            .stdin(std::process::Stdio::null())   // Also suppress stdin
            .spawn()?;
        
        // Show status with retry logic  
        show_detailed_status()?;
        println!("\nPlayer started in background. Use 'ytm stop/pause/next/prev' to control.");
        
        Ok(())
    } else {
        // Run in foreground (blocking)
        Command::new("mpv").args(&args).status()?;
        Ok(())
    }
}

pub fn pause() -> anyhow::Result<()> {
    use serde_json::json;
    mpv::send_mpv_command(json!({"command": ["cycle", "pause"]}))
}

pub fn resume() -> anyhow::Result<()> {
    use serde_json::json;
    mpv::send_mpv_command(json!({"command": ["cycle", "pause"]}))
}

pub fn next() -> anyhow::Result<()> {
    use serde_json::json;
    
    // Get current track position before change
    let current_pos = if let Ok(mut client) = Mpv::connect() {
        client.get_property("playlist-pos-1").ok().flatten().and_then(|v| v.as_i64())
    } else {
        None
    };
    
    mpv::send_mpv_command(json!({"command": ["playlist-next", "force"]}))?;
    
    // Wait for actual track position change (not timing)
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Ok(mut client) = Mpv::connect() {
            if let Ok(Some(new_pos)) = client.get_property("playlist-pos-1").map(|v| v.and_then(|val| val.as_i64())) {
                if Some(new_pos) != current_pos {
                    // Position changed, show status
                    show_detailed_status()?;
                    return Ok(());
                }
            }
        }
    }
    
    // Fallback if position detection fails
    show_detailed_status()?;
    Ok(())
}

pub fn prev() -> anyhow::Result<()> {
    use serde_json::json;
    
    // Get current track position before change
    let current_pos = if let Ok(mut client) = Mpv::connect() {
        client.get_property("playlist-pos-1").ok().flatten().and_then(|v| v.as_i64())
    } else {
        None
    };
    
    mpv::send_mpv_command(json!({"command": ["playlist-prev", "force"]}))?;
    
    // Wait for actual track position change (not timing)
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Ok(mut client) = Mpv::connect() {
            if let Ok(Some(new_pos)) = client.get_property("playlist-pos-1").map(|v| v.and_then(|val| val.as_i64())) {
                if Some(new_pos) != current_pos {
                    // Position changed, show status
                    show_detailed_status()?;
                    return Ok(());
                }
            }
        }
    }
    
    // Fallback if position detection fails
    show_detailed_status()?;
    Ok(())
}

pub fn stop() -> anyhow::Result<()> {
    use serde_json::json;
    if mpv::send_mpv_command(json!({"command": ["stop"]})).is_ok() {
        return Ok(());
    }
    // Fallback to force kill
    mpv::force_kill()
}

/// Debug function to show all available metadata
fn show_all_metadata() -> anyhow::Result<()> {
    let mut mpv_client = Mpv::connect()?;
    
    println!("=== EXPANDED METADATA SEARCH ===");
    
    // Try to get the property list from mpv itself
    if let Ok(Some(property_list)) = mpv_client.get_property("property-list") {
        println!("Available properties: {:?}", property_list);
    }
    
    // Try playlist-specific properties
    let playlist_fields = [
        "playlist", "playlist-current", "chapter-list", "track-list",
        "filename", "stream-open-filename", "path", "stream-path"
    ];
    
    println!("\n--- PLAYLIST/TRACK INFO ---");
    for field in &playlist_fields {
        if let Ok(Some(value)) = mpv_client.get_property(field) {
            println!("{}: {:?}", field, value);
        }
    }
    
    // Try all metadata/ prefixed fields more exhaustively  
    let metadata_fields = [
        "metadata", "metadata/by-key", "filtered-metadata",
        "metadata/list", "track-metadata", "chapter-metadata"
    ];
    
    println!("\n--- METADATA CONTAINERS ---");
    for field in &metadata_fields {
        if let Ok(Some(value)) = mpv_client.get_property(field) {
            println!("{}: {:?}", field, value);
        }
    }
    
    println!("=== END EXPANDED SEARCH ===");
    Ok(())
}

pub fn status() -> anyhow::Result<()> {
    if !mpv::is_running() {
        println!("No player currently running");
        return Ok(());
    }
    

    
    show_detailed_status()
}

pub fn status_live() -> anyhow::Result<()> {
    loop {
        let still_playing = show_status_once()?;
        if !still_playing {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(Some(0.0)), "00:00");
        assert_eq!(format_time(Some(59.0)), "00:59");
        assert_eq!(format_time(Some(60.0)), "01:00");
        assert_eq!(format_time(Some(125.5)), "02:06");
        assert_eq!(format_time(None), "--:--");
    }
}
