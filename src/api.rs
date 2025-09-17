use anyhow::Result;
use reqwest::Client;

use crate::cache::Cache;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SearchResponse {
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SearchItem {
    pub id: ItemId,
    pub snippet: Snippet,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ItemId {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Snippet {
    pub title: String,
    #[serde(rename = "channelTitle")]
    pub channel_title: String,
    pub description: Option<String>,
}

pub struct YouTubeClient {
    api_key: String,
    http: Client,
    cache: Cache,
}

impl YouTubeClient {
    pub fn new(api_key: String, cache: Cache) -> Self {
        Self {
            api_key,
            http: Client::new(),
            cache,
        }
    }

    /// Search YouTube with optional parameters
    pub async fn search(&self, query: &str, max_results: Option<u32>) -> Result<Vec<SearchItem>> {
        let max_results = max_results.unwrap_or(5);
        let search_type = if max_results > 10 { "video,playlist" } else { "video" };
        
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type={}&maxResults={}&key={}",
            query, search_type, max_results, self.api_key
        );

        self.get_search_results_cached(&url).await
    }

    /// Generic cached HTTP GET for YouTube API
    async fn get_search_results_cached(&self, url: &str) -> Result<Vec<SearchItem>> {
        if let Some(cached) = self.cache.get::<Vec<SearchItem>>(url) {
            return Ok(cached);
        }

        let resp: SearchResponse = self.http.get(url).send().await?.json().await?;

        // Filter to only videos and playlists with valid IDs
        let items = resp.items
            .into_iter()
            .filter(|item| item.id.video_id.is_some() || item.id.playlist_id.is_some())
            .collect();

        self.cache.put(url, &items)?;
        Ok(items)
    }
}

/// 🔑 Quick check if API key works
pub async fn validate_key(key: &str) -> bool {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&q=test&type=video&maxResults=1&key={}",
        key
    );

    match Client::new().get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_item_deserialization() {
        let json = r#"{
            "id": {
                "kind": "youtube#video",
                "videoId": "test_video_id"
            },
            "snippet": {
                "title": "Test Video Title",
                "channelTitle": "Test Channel",
                "description": "Test description"
            }
        }"#;

        let item: SearchItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id.video_id, Some("test_video_id".to_string()));
        assert_eq!(item.snippet.title, "Test Video Title");
        assert_eq!(item.snippet.channel_title, "Test Channel");
    }

    #[test]
    fn test_playlist_item_deserialization() {
        let json = r#"{
            "id": {
                "kind": "youtube#playlist",
                "playlistId": "test_playlist_id"
            },
            "snippet": {
                "title": "Test Playlist",
                "channelTitle": "Test Channel"
            }
        }"#;

        let item: SearchItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id.playlist_id, Some("test_playlist_id".to_string()));
        assert_eq!(item.id.video_id, None);
        assert_eq!(item.snippet.title, "Test Playlist");
    }

    #[test] 
    fn test_search_response_deserialization() {
        let json = r#"{
            "items": [
                {
                    "id": {
                        "kind": "youtube#video", 
                        "videoId": "video1"
                    },
                    "snippet": {
                        "title": "Video 1",
                        "channelTitle": "Channel 1"
                    }
                }
            ]
        }"#;

        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].snippet.title, "Video 1");
    }
}
