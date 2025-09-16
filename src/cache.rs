use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

#[derive(Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub timestamp: u64,
    pub data: T,
}

pub struct Cache {
    dir: PathBuf,
    ttl: Duration,
}

impl Cache {
    pub fn new<P: AsRef<Path>>(dir: P, ttl: Duration) -> anyhow::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir: dir.as_ref().to_path_buf(),
            ttl,
        })
    }

    fn key_path(&self, key: &str) -> PathBuf {
        let safe = STANDARD.encode(key);
        self.dir.join(format!("{}.json", safe))
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let path = self.key_path(key);
        let raw = fs::read(path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_slice(&raw).ok()?;

        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs();
        if now - entry.timestamp <= self.ttl.as_secs() {
            Some(entry.data)
        } else {
            None
        }
    }

    pub fn put<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()> {
        let entry = CacheEntry {
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            data,
        };
        let raw = serde_json::to_vec(&entry)?;
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, raw)?;
        Ok(())
    }
}
