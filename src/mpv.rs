use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use std::path::PathBuf;

use anyhow::Result;
use serde_json::{json, Value};

/// Get the path for MPV Unix socket
fn mpv_socket() -> PathBuf {
    PathBuf::from("/tmp/ytm-mpv.sock")
}

/// Get the path for MPV PID file
fn mpv_pid_file() -> PathBuf {
    PathBuf::from("/tmp/ytm-mpv.pid")
}

pub struct Mpv {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
}

impl Mpv {
    /// Connect to MPV IPC socket
    pub fn connect() -> Result<Self> {
        let stream = UnixStream::connect(mpv_socket())?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    /// Send a command to MPV
    pub fn send_command(&mut self, cmd: Value) -> Result<()> {
        let line = serde_json::to_string(&cmd)? + "\n";
        self.stream.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Get a property value from MPV
    #[allow(dead_code)] // Part of public API, may be used in future
    pub fn get_property(&mut self, property: &str) -> Result<Option<Value>> {
        self.send_command(json!({"command": ["get_property", property]}))?;
        
        let mut buf = String::new();
        if self.reader.read_line(&mut buf).is_ok() {
            if let Ok(val) = serde_json::from_str::<Value>(&buf) {
                return Ok(val.get("data").cloned());
            }
        }
        Ok(None)
    }

    /// Get multiple properties at once
    pub fn get_status(&mut self) -> Result<MpvStatus> {
        // Send all property requests
        self.send_command(json!({"command": ["get_property", "media-title"]}))?;
        self.send_command(json!({"command": ["get_property", "time-pos"]}))?;
        self.send_command(json!({"command": ["get_property", "duration"]}))?;

        let mut title = None;
        let mut position = None;
        let mut duration = None;

        // Read responses
        for _ in 0..3 {
            let mut buf = String::new();
            if self.reader.read_line(&mut buf).is_err() {
                break;
            }
            if let Ok(val) = serde_json::from_str::<Value>(&buf) {
                if buf.contains("media-title") {
                    title = val["data"].as_str().map(|s| s.to_string());
                } else if buf.contains("time-pos") {
                    position = val["data"].as_f64();
                } else if buf.contains("duration") {
                    duration = val["data"].as_f64();
                }
            }
        }

        Ok(MpvStatus {
            title,
            position,
            duration,
        })
    }

    /// Toggle pause state
    #[allow(dead_code)] // Part of public API
    pub fn toggle_pause(&mut self) -> Result<()> {
        self.send_command(json!({"command": ["cycle", "pause"]}))
    }

    /// Go to next track
    #[allow(dead_code)] // Part of public API
    pub fn next(&mut self) -> Result<()> {
        self.send_command(json!({"command": ["playlist-next", "force"]}))
    }

    /// Go to previous track
    #[allow(dead_code)] // Part of public API
    pub fn prev(&mut self) -> Result<()> {
        self.send_command(json!({"command": ["playlist-prev", "force"]}))
    }

    /// Stop playback
    #[allow(dead_code)] // Part of public API
    pub fn stop(&mut self) -> Result<()> {
        self.send_command(json!({"command": ["stop"]}))
    }
}

pub struct MpvStatus {
    pub title: Option<String>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

/// Send a one-off command to MPV (convenience function)
pub fn send_mpv_command(cmd: Value) -> Result<()> {
    let mut stream = UnixStream::connect(mpv_socket())?;
    let line = serde_json::to_string(&cmd)? + "\n";
    stream.write_all(line.as_bytes())?;
    Ok(())
}

/// Check if MPV is running
#[allow(dead_code)] // Utility function for future use
pub fn is_running() -> bool {
    UnixStream::connect(mpv_socket()).is_ok()
}

/// Kill MPV process using PID file (fallback method)
pub fn force_kill() -> Result<()> {
    if let Ok(pid_str) = fs::read_to_string(mpv_pid_file()) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
        }
    }
    Ok(())
}
