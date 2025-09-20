use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rustypipe::{
    client::RustyPipe,
    model::{PlaylistItem, VideoItem, YouTubeItem},
};

use crate::cache::Cache;

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
    pipe: RustyPipe,
    cache: Cache,
}

impl YouTubeClient {
    pub fn new<P: AsRef<Path>>(storage_root: P, cache: Cache) -> Result<Self> {
        let storage_dir = storage_root.as_ref().join("rustypipe");
        fs::create_dir_all(&storage_dir).with_context(|| {
            format!("failed to create rustypipe cache dir at {:?}", storage_dir)
        })?;

        let pipe = RustyPipe::builder()
            .storage_dir(storage_dir)
            .build()
            .context("failed to initialize RustyPipe")?;

        Ok(Self { pipe, cache })
    }

    pub async fn search(&self, query: &str, max_results: Option<u32>) -> Result<Vec<SearchItem>> {
        let max_results = max_results.unwrap_or(5).clamp(1, 50) as usize;
        let cache_key = format!("ytm::search::{max_results}::{query}");

        if let Some(cached) = self.cache.get::<Vec<SearchItem>>(&cache_key) {
            return Ok(cached);
        }

        let pipe_query = self.pipe.query();
        let search_result = pipe_query
            .search::<YouTubeItem, _>(query)
            .await
            .context("rustypipe search request failed")?;

        let mut paginator = search_result.items;
        if paginator.items.len() < max_results {
            paginator
                .extend_limit(pipe_query.clone(), max_results)
                .await
                .context("failed to extend search results")?;
        }

        let items: Vec<SearchItem> = paginator
            .items
            .into_iter()
            .filter_map(|item| SearchItem::try_from(item).ok())
            .take(max_results)
            .collect();

        if !items.is_empty() {
            // Cache only on success so transient errors don't poison the cache
            self.cache
                .put(&cache_key, &items)
                .context("failed to cache search results")?;
        }

        Ok(items)
    }
}

impl TryFrom<YouTubeItem> for SearchItem {
    type Error = ();

    fn try_from(value: YouTubeItem) -> Result<Self, Self::Error> {
        match value {
            YouTubeItem::Video(video) => Ok(video.into()),
            YouTubeItem::Playlist(playlist) => Ok(playlist.into()),
            YouTubeItem::Channel(_) => Err(()),
        }
    }
}

impl From<VideoItem> for SearchItem {
    fn from(video: VideoItem) -> Self {
        let channel_title = video
            .channel
            .map(|channel| channel.name)
            .unwrap_or_else(|| "Unknown channel".to_string());

        SearchItem {
            id: ItemId {
                kind: "youtube#video".to_string(),
                video_id: Some(video.id),
                playlist_id: None,
            },
            snippet: Snippet {
                title: video.name,
                channel_title,
                description: video.short_description,
            },
        }
    }
}

impl From<PlaylistItem> for SearchItem {
    fn from(playlist: PlaylistItem) -> Self {
        let channel_title = playlist
            .channel
            .map(|channel| channel.name)
            .unwrap_or_else(|| "Unknown channel".to_string());

        let description = playlist
            .video_count
            .map(|count| format!("{} videos", count));

        SearchItem {
            id: ItemId {
                kind: "youtube#playlist".to_string(),
                video_id: None,
                playlist_id: Some(playlist.id),
            },
            snippet: Snippet {
                title: playlist.name,
                channel_title,
                description,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustypipe::model::{ChannelItem, PlaylistItem, VideoItem, YouTubeItem};
    use serde_json::json;

    #[test]
    fn maps_video_items() {
        let json_item = json!({
            "id": "video123",
            "name": "Fantastic Track",
            "duration": 240,
            "thumbnail": [
                {
                    "url": "http://example.com/thumb.jpg",
                    "width": 1280,
                    "height": 720
                }
            ],
            "channel": {
                "id": "channelABC",
                "name": "Great Artist",
                "avatar": [],
                "verification": "verified",
                "subscriber_count": null
            },
            "publish_date": null,
            "publish_date_txt": null,
            "view_count": 1000,
            "is_live": false,
            "is_short": false,
            "is_upcoming": false,
            "short_description": "A lovely song"
        });

        let item: VideoItem = serde_json::from_value(json_item).unwrap();

        let mapped: SearchItem = item.into();
        assert_eq!(mapped.id.video_id.as_deref(), Some("video123"));
        assert_eq!(mapped.snippet.title, "Fantastic Track");
        assert_eq!(mapped.snippet.channel_title, "Great Artist");
        assert_eq!(mapped.snippet.description.as_deref(), Some("A lovely song"));
    }

    #[test]
    fn maps_playlist_items() {
        let json_item = json!({
            "id": "playlist456",
            "name": "Chill Mix",
            "thumbnail": [],
            "channel": {
                "id": "channelXYZ",
                "name": "DJ Test",
                "avatar": [],
                "verification": "none",
                "subscriber_count": null
            },
            "video_count": 42
        });

        let item: PlaylistItem = serde_json::from_value(json_item).unwrap();

        let mapped: SearchItem = item.into();
        assert_eq!(mapped.id.playlist_id.as_deref(), Some("playlist456"));
        assert_eq!(mapped.snippet.title, "Chill Mix");
        assert_eq!(mapped.snippet.channel_title, "DJ Test");
        assert_eq!(mapped.snippet.description.as_deref(), Some("42 videos"));
    }

    #[test]
    fn drops_channel_items() {
        let channel_json = json!({
            "id": "channel1",
            "name": "Only Channels",
            "handle": null,
            "avatar": [],
            "verification": "none",
            "subscriber_count": 10,
            "short_description": "About"
        });

        let channel: ChannelItem = serde_json::from_value(channel_json).unwrap();
        let item = YouTubeItem::Channel(channel);

        assert!(SearchItem::try_from(item).is_err());
    }
}
