use std::collections::HashSet;

use serde_json::{Map, Value};

use super::Spotify;

#[derive(Default)]
pub(super) struct Walker {
    pub(super) queries: Vec<String>,
    seen: HashSet<String>,
}

impl Walker {
    const TITLE_KEYS: &[&str] = &["name", "title", "trackName", "displayName"];
    const ARTIST_KEYS: &[&str] = &["artistName", "artist", "byline", "subtitle", "ownerName"];

    /// `title_hint` and `artist_hint` come from a parent object, for tracks
    /// that only carry half of it themselves.
    pub(super) fn walk(
        &mut self,
        value: &Value,
        title_hint: Option<String>,
        artist_hint: Option<String>,
    ) {
        match value {
            Value::Object(obj) => self.walk_object(obj, title_hint, artist_hint),
            Value::Array(arr) => {
                for child in arr {
                    self.walk(child, title_hint.clone(), artist_hint.clone());
                }
            }
            // Some fields hold more JSON as a string.
            Value::String(s) => {
                let s = s.trim();

                if ((s.starts_with('{') && s.ends_with('}'))
                    || (s.starts_with('[') && s.ends_with(']')))
                    && let Ok(parsed) = serde_json::from_str::<Value>(s)
                {
                    self.walk(&parsed, title_hint, artist_hint);
                }
            }
            _ => {}
        }
    }

    /// Title and artist of `obj`, when it is a track on its own.
    fn track_info(obj: &Map<String, Value>) -> Option<(String, String)> {
        let title = Self::string_from_keys(obj, Self::TITLE_KEYS)?;
        let artist = Self::first_artist(obj)?;

        let looks_like_track =
            Self::is_trackish(obj) || obj.contains_key("artists") || obj.contains_key("album");

        looks_like_track.then_some((title, artist))
    }

    fn is_trackish(obj: &Map<String, Value>) -> bool {
        obj.get("uri")
            .and_then(Value::as_str)
            .is_some_and(|u| u.contains("spotify:track:") || u.contains("/track/"))
            || obj.get("type").and_then(Value::as_str) == Some("track")
            || obj.contains_key("duration_ms")
    }

    /// The first entry of `artists`, else an artist-like field on `obj` itself.
    fn first_artist(obj: &Map<String, Value>) -> Option<String> {
        obj.get("artists")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|first| {
                first
                    .get("name")
                    .or_else(|| first.get("title"))
                    .or_else(|| first.get("artistName"))
            })
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
            .or_else(|| Self::string_from_keys(obj, Self::ARTIST_KEYS))
    }

    fn string_from_keys(obj: &Map<String, Value>, keys: &[&str]) -> Option<String> {
        keys.iter()
            .filter_map(|key| obj.get(*key).and_then(Value::as_str))
            .map(str::trim)
            .find(|s| !s.is_empty())
            .map(str::to_string)
    }

    fn walk_object(
        &mut self,
        obj: &Map<String, Value>,
        title_hint: Option<String>,
        artist_hint: Option<String>,
    ) {
        // Prefer walking the current entity branch first.
        if let Some(entity) = obj.get("entity") {
            self.walk(entity, None, None);
        }

        for key in ["items", "tracks", "track"] {
            if let Some(child) = obj.get(key) {
                self.walk(child, title_hint.clone(), artist_hint.clone());
            }
        }

        let title = Self::string_from_keys(obj, Self::TITLE_KEYS).or(title_hint);
        let artist = Self::first_artist(obj).or(artist_hint);

        if let Some((title, artist)) = Self::track_info(obj) {
            self.push(&title, &artist);
        } else if let (Some(title), Some(artist)) = (&title, &artist)
            && Self::is_trackish(obj)
        {
            self.push(title, artist);
        }

        for child in obj.values() {
            self.walk(child, title.clone(), artist.clone());
        }
    }

    fn push(&mut self, title: &str, artist: &str) {
        // Spotify separates multiple artists with a literal `&nbsp;`.
        let artist = artist.replace("&nbsp;", " ");
        let title = title.trim();
        let artist = artist.trim();

        if title.is_empty() || artist.is_empty() || title == "Spotify" {
            return;
        }

        let query = format!("{}{title} {artist}", Spotify::SEARCH_PREFIX);

        if self.seen.insert(query.clone()) {
            self.queries.push(query);
        }
    }
}
