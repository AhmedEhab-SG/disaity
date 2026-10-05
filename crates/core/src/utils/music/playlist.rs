use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use tokio::process::Command;

use crate::{binaries::Binaries, errors::Error};

use super::{Batch, resolver::TrackResolver, spotify::Spotify};

/// One song of a playlist. `meta` is set when the playlist listing already
/// carried it, so the song can be queued without its own yt-dlp lookup.
pub(super) struct Playlist {
    query: String,
    meta: Option<ListedMeta>,
}

pub(super) struct ListedMeta {
    pub(super) title: String,
    pub(super) duration: Option<Duration>,
    pub(super) author: Option<String>,
    pub(super) thumbnail: Option<String>,
}

impl Playlist {
    const LIST_INDICATORS: [&str; 3] = ["list=", "/sets/", "/playlist/"];

    /// Splits a playlist into tracks that can be queued now and queries that still
    /// need a lookup.
    pub(super) async fn load(resolver: &TrackResolver, url: &str) -> Result<Batch, Error> {
        let mut ready = Vec::new();
        let mut pending = Vec::new();

        for entry in Self::entries(&resolver.http, url).await? {
            // Skip nested playlists
            if Self::is_list(&entry.query) {
                continue;
            }

            match entry.meta {
                // Once one song has to wait for a lookup, every later one waits too,
                // so the queue keeps the playlist order.
                Some(meta) if pending.is_empty() => ready.push(resolver.listed(entry.query, meta)?),
                _ => pending.push(entry.query),
            }
        }

        // Nothing came with metadata (Spotify, some SoundCloud sets): resolve songs
        // until one works so playback can start right away.
        let mut pending = pending.into_iter();
        while ready.is_empty() {
            let Some(query) = pending.next() else { break };
            if let Ok(track) = resolver.resolve(query).await {
                ready.push(track);
            }
        }

        Ok(Batch {
            ready,
            pending: pending.collect(),
        })
    }

    pub(super) fn is_list(url: &str) -> bool {
        Self::LIST_INDICATORS
            .iter()
            .any(|indicator| url.contains(indicator))
    }

    async fn entries(http: &Client, url: &str) -> Result<Vec<Self>, Error> {
        if Spotify::is_list(url) {
            let queries = Spotify::playlist_queries(http, url).await?;
            return Ok(queries
                .into_iter()
                .map(|query| Self { query, meta: None })
                .collect());
        }

        let binaries = Binaries::get()?;

        // One yt-dlp run lists the whole playlist, with each song's title,
        // duration, channel and thumbnail.
        let output = Command::new(binaries.ytdlp.path())
            .args(binaries.ffmpeg_args())
            .args([
                "--flat-playlist",
                "--yes-playlist",
                "--ignore-config",
                "--no-warnings",
                "--dump-json",
                url,
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|entry| Self::parse_entry(&entry))
            .collect())
    }

    /// One `--dump-json` line of a flat playlist.
    fn parse_entry(entry: &Value) -> Option<Self> {
        // yt-dlp often sends `"channel": null`, so try each key until one is a string.
        let str_field = |keys: &[&str]| {
            keys.iter()
                .find_map(|key| entry.get(*key).and_then(Value::as_str))
                .map(str::to_string)
        };

        let query = str_field(&["webpage_url", "url"])?;
        if !query.starts_with("http") {
            return None;
        }

        let meta = str_field(&["title"]).map(|title| ListedMeta {
            title,
            duration: entry
                .get("duration")
                .and_then(Value::as_f64)
                .map(Duration::from_secs_f64),
            author: str_field(&["channel", "uploader"]),
            thumbnail: entry
                .get("thumbnails")
                .and_then(Value::as_array)
                .and_then(|thumbs| thumbs.last())
                .and_then(|thumb| thumb.get("url"))
                .and_then(Value::as_str)
                .map(str::to_string),
        });

        // Private and deleted videos stay listed but can't be played.
        if meta
            .as_ref()
            .is_some_and(|m| m.title == "[Private video]" || m.title == "[Deleted video]")
        {
            return None;
        }

        Some(Self { query, meta })
    }
}
