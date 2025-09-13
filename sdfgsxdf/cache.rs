// src/cache.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Wrapper around cached query results
#[derive(Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub timestamp: u64, // epoch secs
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
        // sanitize query into filename (very basic)
        let safe = base64::encode(key);
        self.dir.join(format!("{}.json", safe))
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let path = self.key_path(key);
        let raw = fs::read(path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_slice(&raw).ok()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - entry.timestamp <= self.ttl.as_secs() {
            Some(entry.data)
        } else {
            None // stale
        }
    }

    pub fn put<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()> {
        let entry = CacheEntry {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            data,
        };
        let raw = serde_json::to_vec(&entry)?;
        fs::write(self.key_path(key), raw)?;
        Ok(())
    }
}
