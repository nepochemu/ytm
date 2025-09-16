use std::fs;
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::os::unix::net::UnixStream;
use serde_json::json;

const PID_FILE: &str = "/tmp/ytm-mpv.pid";
const SOCK_PATH: &str = "/tmp/ytm-mpv.sock";

// Replace your existing play/search integration here
pub fn search_and_play(query: &str, _api: Option<String>, background: bool) -> anyhow::Result<()> {
    // TODO: resolve query → video_url (reuse your existing API logic)
    let url = query; // placeholder

    play(url, background)
}

pub fn play(url: &str, background: bool) -> anyhow::Result<()> {
    if background {
        let _ = fs::remove_file(SOCK_PATH);

        let child = Command::new("mpv")
            .args([
                "--no-terminal",
                "--idle=yes",
                &format!("--input-ipc-server={}", SOCK_PATH),
                url,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        fs::write(PID_FILE, child.id().to_string())?;
        println!("▶ Playing in background: {}", url);
        println!("Use: ytm pause | next | prev | stop");
    } else {
        Command::new("mpv").args([url]).status()?;
    }
    Ok(())
}

// ---- Controls ----

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

// ---- Status ----

pub fn status() -> anyhow::Result<()> {
    loop {
        let still_playing = show_status_once()?;
        if !still_playing {
            println!(); // newline after done
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    Ok(())
}

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
        let pos_str = pos.map(|p| format!("{:.0}:{:02}", (p/60.0).floor(), (p%60.0).round()))
                         .unwrap_or_else(|| "--:--".to_string());
        let dur_str = dur.map(|d| format!("{:.0}:{:02}", (d/60.0).floor(), (d%60.0).round()))
                         .unwrap_or_else(|| "--:--".to_string());
        print!("\r▶ {}  [{} / {}]   ", t, pos_str, dur_str);
        std::io::stdout().flush().ok();
        Ok(true)
    } else {
        println!("(no track playing)");
        Ok(false)
    }
}
