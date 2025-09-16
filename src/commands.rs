use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};

use serde_json::json;

use crate::api::YouTubeClient;
use crate::cache::Cache;
use crate::config::Config;

const PID_FILE: &str = "/tmp/ytm-mpv.pid";
const SOCK_PATH: &str = "/tmp/ytm-mpv.sock";

fn show_status_once() -> anyhow::Result<bool> {
    let mut stream = match UnixStream::connect(SOCK_PATH) {
        Ok(s) => s,
        Err(_) => {
            println!("(player not running)");
            return Ok(false);
        }
    };
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut send = |cmd: serde_json::Value| -> anyhow::Result<()> {
        let line = serde_json::to_string(&cmd)? + "\n";
        stream.write_all(line.as_bytes())?;
        Ok(())
    };

    send(json!({"command": ["get_property", "media-title"]}))?;
    send(json!({"command": ["get_property", "time-pos"]}))?;
    send(json!({"command": ["get_property", "duration"]}))?;

    let mut title = None;
    let mut pos = None;
    let mut dur = None;

    for _ in 0..3 {
        let mut buf = String::new();
        if reader.read_line(&mut buf).is_err() {
            break;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&buf) {
            if buf.contains("media-title") {
                title = val["data"].as_str().map(|s| s.to_string());
            } else if buf.contains("time-pos") {
                pos = val["data"].as_f64();
            } else if buf.contains("duration") {
                dur = val["data"].as_f64();
            }
        }
    }

    if let Some(t) = title {
        let pos_str = pos.map(|p| format!("{:.0}:{:02}", (p/60.0).floor(), (p%60.0).round())).unwrap_or_else(|| "--:--".to_string());
        let dur_str = dur.map(|d| format!("{:.0}:{:02}", (d/60.0).floor(), (d%60.0).round())).unwrap_or_else(|| "--:--".to_string());
        print!("\r▶ {}  [{} / {}]   ", t, pos_str, dur_str);
        std::io::stdout().flush().ok();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Search YouTube, pick first result, and play
pub async fn search_and_play(query: &str, api: Option<String>, background: bool, no_video: bool) -> anyhow::Result<()> {
    let mut cfg = Config::load()?;
    if let Some(api_key) = api {
        cfg.api_key = Some(api_key);
        cfg.save()?;
    }
    let key = cfg.ensure_api_key().await?;

    let cache_dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("ytm");
    let cache = Cache::new(cache_dir, std::time::Duration::from_secs(3600))?;
    let client = YouTubeClient::new(key, cache);

    // Fetch 50 results
    let results = client.search_with_max_results(query, 50).await?;
    if results.is_empty() {
        return Err(anyhow::anyhow!("No results for '{}'", query));
    }

    // Format for fzf: concise info, add [playlist] if needed, hide ID from display
    let mut lines = Vec::new();
    let mut ids = Vec::new();
    let mut is_playlist_vec = Vec::new();
    for v in &results {
        let title = v.snippet["title"].as_str().unwrap_or("");
        let channel = v.snippet["channelTitle"].as_str().unwrap_or("");
        let (id_str, label, is_playlist) = if let Some(playlist_id) = v.id.get("playlistId").and_then(|v| v.as_str()) {
            (playlist_id, " [playlist]", true)
        } else if let Some(video_id) = v.id.get("videoId").and_then(|v| v.as_str()) {
            (video_id, "", false)
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
    play(&url, background, no_video)?;
    if background {
        // Show status once, then release terminal
        let _ = show_status_once();
    }
    Ok(())
}

/// Start mpv either foreground or background with IPC enabled
pub fn play(url: &str, background: bool, no_video: bool) -> anyhow::Result<()> {
    let mut args = Vec::new();
    if no_video {
        args.push("--no-video");
    }
    if background {
        let _ = fs::remove_file(SOCK_PATH);
        let ipc_arg = format!("--input-ipc-server={}", SOCK_PATH);
        args.extend([
            "--no-terminal",
            "--idle=yes",
            &ipc_arg,
            url,
        ]);
        let child = Command::new("mpv")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        fs::write(PID_FILE, child.id().to_string())?;
    } else {
        args.push(url);
        Command::new("mpv").args(&args).status()?;
    }
    Ok(())
}

fn send_mpv(cmd: serde_json::Value) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(SOCK_PATH)?;
    let line = serde_json::to_string(&cmd)? + "\n";
    stream.write_all(line.as_bytes())?;
    Ok(())
}

pub fn pause() -> anyhow::Result<()> {
    send_mpv(json!({"command": ["cycle", "pause"]}))
}

pub fn next() -> anyhow::Result<()> {
    send_mpv(json!({"command": ["playlist-next", "force"]}))
}

pub fn prev() -> anyhow::Result<()> {
    send_mpv(json!({"command": ["playlist-prev", "force"]}))
}

pub fn stop() -> anyhow::Result<()> {
    if let Ok(()) = send_mpv(json!({"command": ["stop"]})) {
        return Ok(());
    }
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
        }
    }
    Ok(())
}

pub fn status() -> anyhow::Result<()> {
    loop {
        let still_playing = show_status_once()?;
        if !still_playing {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    Ok(())
}
