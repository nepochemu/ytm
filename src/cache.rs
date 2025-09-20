use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub timestamp_millis: u128,
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

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_millis();
        if now - entry.timestamp_millis <= self.ttl.as_millis() {
            Some(entry.data)
        } else {
            None
        }
    }

    pub fn put<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()> {
        let entry = CacheEntry {
            timestamp_millis: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        value: String,
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = tempdir().unwrap();
        let cache = Cache::new(temp_dir.path(), Duration::from_secs(60)).unwrap();

        let test_data = TestData {
            value: "test_value".to_string(),
        };

        // Put data in cache
        cache.put("test_key", &test_data).unwrap();

        // Get data from cache
        let retrieved: Option<TestData> = cache.get("test_key");
        assert_eq!(retrieved, Some(test_data));
    }

    #[test]
    fn test_cache_expiry() {
        let temp_dir = tempdir().unwrap();
        // Very short TTL for testing
        let cache = Cache::new(temp_dir.path(), Duration::from_millis(50)).unwrap();

        let test_data = TestData {
            value: "expired_test".to_string(),
        };

        // Put data in cache
        cache.put("expiry_key", &test_data).unwrap();

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(100));

        // Should return None due to expiry
        let retrieved: Option<TestData> = cache.get("expiry_key");
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_cache_nonexistent_key() {
        let temp_dir = tempdir().unwrap();
        let cache = Cache::new(temp_dir.path(), Duration::from_secs(60)).unwrap();

        let retrieved: Option<TestData> = cache.get("nonexistent_key");
        assert_eq!(retrieved, None);
    }
}
