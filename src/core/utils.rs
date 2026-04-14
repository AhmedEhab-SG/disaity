use scraper::{Html, Selector};
use serde_json::{Map, Value};
use serenity::{all::ReactionType, async_trait};
use songbird::{
    Call,
    input::{Input, YoutubeDl},
    tracks::Track,
};
use std::{collections::HashSet, sync::Arc};
use tokio::{process::Command, sync::Mutex};

use crate::{
    config::interactions::Provider,
    core::{context::Context, error::Error},
    handlers::{SongMetadata, register_all},
};

pub struct Utils<'a> {
    pub ctx: Context<'a>,
}

#[async_trait]
pub trait ReactionUtils {
    async fn add_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error>;

    async fn delete_self_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error>;

    async fn delete_all_self_reactions(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait VoiceUtils {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error>;
}

#[async_trait]
trait ExtractorUtils {
    async fn spotify_extractor(&self, url: &str) -> Result<String, Error>;
    async fn playlist_extractor(&self, url: &str) -> Result<Vec<String>, Error>;
    async fn spotify_playlist_extractor(&self, url: &str) -> Result<Vec<String>, Error>;
}

#[async_trait]
trait ProviderUtils {
    async fn youtube_provider(&self, song: String) -> Result<(Track, SongMetadata), Error>;
}

#[async_trait]
impl ReactionUtils for Utils<'_> {
    async fn add_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            for emoji in emojis {
                p_ctx.msg.react(self.ctx, emoji.clone().into()).await?;
            }
        }
        Ok(())
    }

    async fn delete_self_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            let target_emojis: Vec<ReactionType> =
                emojis.iter().map(|e| e.clone().into()).collect();

            let updated_msg = self
                .ctx
                .channel_id()
                .message(self.ctx, p_ctx.msg.id)
                .await?;

            for reaction in &updated_msg.reactions {
                if !reaction.me {
                    continue;
                }

                if target_emojis.contains(&reaction.reaction_type) {
                    updated_msg
                        .delete_reaction(self.ctx, None, reaction.reaction_type.clone())
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn delete_all_self_reactions(&self) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            let updated_msg = self
                .ctx
                .channel_id()
                .message(self.ctx, p_ctx.msg.id)
                .await?;

            for reaction in &updated_msg.reactions {
                if !reaction.me {
                    continue;
                }

                updated_msg
                    .delete_reaction(self.ctx, None, reaction.reaction_type.clone())
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl VoiceUtils for Utils<'_> {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error> {
        let serenity_context = self.ctx.serenity_context();
        let guild_id = self
            .ctx
            .guild_id()
            .ok_or("This command only works in servers.")?;
        let voice_channel_id = self
            .ctx
            .guild()
            .and_then(|g| {
                g.voice_states
                    .get(&self.ctx.author().id)
                    .and_then(|vs| vs.channel_id)
            })
            .ok_or("You must be in a voice channel!")?;
        let manager = songbird::get(serenity_context)
            .await
            .ok_or("Failed to mount songbird")?;

        let (call, is_new_call) = if let Some(exisiting_call) = manager.get(guild_id) {
            let mut call_lock = exisiting_call.lock().await;
            call_lock.join(voice_channel_id).await.ok();
            drop(call_lock);

            (exisiting_call, false)
        } else {
            let new_call = manager.join(guild_id, voice_channel_id).await?;
            (new_call, true)
        };

        if is_new_call {
            let mut call_lock = call.lock().await;

            register_all(
                &mut call_lock,
                call.clone(),
                guild_id,
                self.ctx.channel_id(),
                serenity_context.http.clone(),
                manager,
                serenity_context.cache.clone(),
            )
            .await;
        }

        Ok(call)
    }
}

#[async_trait]
impl ExtractorUtils for Utils<'_> {
    async fn spotify_extractor(&self, url: &str) -> Result<String, Error> {
        let res = self.ctx.data().http.get(url).send().await?.text().await?;

        let title = res
            .find("<title>")
            .map(|start| start + 7)
            .and_then(|start_idx| {
                res[start_idx..]
                    .find("</title>")
                    .map(|end_idx| &res[start_idx..start_idx + end_idx])
            })
            .ok_or("failed to get extract the song title")?
            .replace(" - song and lyrics by", "")
            .replace(" | Spotify", "");

        Ok(title)
    }

    async fn spotify_playlist_extractor(&self, url: &str) -> Result<Vec<String>, Error> {
        let clean_url = url.split('?').next().unwrap_or(url).trim_end_matches('/');

        let embed_url = if clean_url.contains("/embed/") {
            clean_url.to_string()
        } else {
            clean_url.replace("open.spotify.com/", "open.spotify.com/embed/")
        };

        let client = &self.ctx.data().http;
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let mut html_text = None;

        // 1. Fetch the HTML
        for candidate in [embed_url.as_str(), clean_url] {
            if let Ok(resp) = client
                .get(candidate)
                .header("User-Agent", ua)
                .header("Accept-Language", "en-US,en;q=0.9")
                .send()
                .await
            {
                if let Ok(text) = resp.text().await {
                    if !text.is_empty() {
                        html_text = Some(text);
                        break;
                    }
                }
            }
        }

        let html_text = html_text.ok_or("Failed to fetch Spotify page")?;

        // 2. Parse HTML and extract JSON using `scraper`
        let document = Html::parse_document(&html_text);
        let selector =
            Selector::parse("script#__NEXT_DATA__").map_err(|_| "Invalid CSS selector")?;

        let json_str = document
            .select(&selector)
            .next()
            .map(|element| element.inner_html())
            .ok_or_else(|| {
                println!(
                    "--- RAW HTML START ---\n{}\n--- RAW HTML END ---",
                    &html_text[..html_text.len().min(1000)]
                );
                "Could not find script#__NEXT_DATA__ in Spotify HTML"
            })?;

        let root: Value = serde_json::from_str(&json_str)?;

        let state_data = root
            .get("props")
            .and_then(|p| p.get("pageProps"))
            .and_then(|pp| pp.get("state"))
            .and_then(|s| s.get("data"))
            .ok_or("Spotify state.data missing")?;

        // 3. Helper Functions
        fn push_query(
            title: &str,
            artist: &str,
            out: &mut Vec<String>,
            seen: &mut HashSet<String>,
        ) {
            let title = title.trim();
            let artist = artist.trim();

            if title.is_empty() || artist.is_empty() || title == "Spotify" {
                return;
            }

            let query = format!("spotifysearch:{} {}", title, artist);
            if seen.insert(query.clone()) {
                out.push(query);
            }
        }

        fn string_from_keys(obj: &Map<String, Value>, keys: &[&str]) -> Option<String> {
            for key in keys {
                if let Some(s) = obj.get(*key).and_then(|v| v.as_str()) {
                    if !s.trim().is_empty() {
                        return Some(s.trim().to_string());
                    }
                }
            }
            None
        }

        fn extract_track_info(obj: &Map<String, Value>) -> Option<(String, String)> {
            let title = string_from_keys(obj, &["name", "title", "trackName", "displayName"])?;

            let artist = obj
                .get("artists")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|first| {
                    first
                        .get("name")
                        .or_else(|| first.get("title"))
                        .or_else(|| first.get("artistName"))
                })
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .or_else(|| {
                    string_from_keys(
                        obj,
                        &["artistName", "artist", "byline", "subtitle", "ownerName"],
                    )
                })?;

            let looks_like_track = obj
                .get("uri")
                .and_then(|v| v.as_str())
                .map(|u| u.contains("spotify:track:") || u.contains("/track/"))
                .unwrap_or(false)
                || obj.get("type").and_then(|v| v.as_str()) == Some("track")
                || obj.contains_key("duration_ms")
                || obj.contains_key("artists")
                || obj.contains_key("album");

            if looks_like_track {
                Some((title, artist))
            } else {
                None
            }
        }

        fn walk(
            value: &Value,
            out: &mut Vec<String>,
            seen: &mut HashSet<String>,
            title_hint: Option<String>,
            artist_hint: Option<String>,
        ) {
            match value {
                Value::Object(obj) => {
                    // Prefer walking the current entity branch first.
                    if let Some(entity) = obj.get("entity") {
                        walk(entity, out, seen, None, None);
                    }

                    if let Some(items) = obj.get("items") {
                        walk(items, out, seen, title_hint.clone(), artist_hint.clone());
                    }

                    if let Some(tracks) = obj.get("tracks") {
                        walk(tracks, out, seen, title_hint.clone(), artist_hint.clone());
                    }

                    if let Some(track) = obj.get("track") {
                        walk(track, out, seen, title_hint.clone(), artist_hint.clone());
                    }

                    let local_title =
                        string_from_keys(obj, &["name", "title", "trackName", "displayName"])
                            .or(title_hint.clone());

                    let local_artist = obj
                        .get("artists")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|first| {
                            first
                                .get("name")
                                .or_else(|| first.get("title"))
                                .or_else(|| first.get("artistName"))
                        })
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_string())
                        .or_else(|| {
                            string_from_keys(
                                obj,
                                &["artistName", "artist", "byline", "subtitle", "ownerName"],
                            )
                        })
                        .or(artist_hint.clone());

                    if let Some((title, artist)) = extract_track_info(obj) {
                        push_query(&title, &artist, out, seen);
                    } else if let (Some(t), Some(a)) = (&local_title, &local_artist) {
                        let is_trackish = obj
                            .get("uri")
                            .and_then(|v| v.as_str())
                            .map(|u| u.contains("spotify:track:") || u.contains("/track/"))
                            .unwrap_or(false)
                            || obj.get("type").and_then(|v| v.as_str()) == Some("track")
                            || obj.contains_key("duration_ms");

                        if is_trackish {
                            push_query(t, a, out, seen);
                        }
                    }

                    for child in obj.values() {
                        walk(child, out, seen, local_title.clone(), local_artist.clone());
                    }
                }
                Value::Array(arr) => {
                    for child in arr {
                        walk(child, out, seen, title_hint.clone(), artist_hint.clone());
                    }
                }
                Value::String(s) => {
                    let s = s.trim();

                    if (s.starts_with('{') && s.ends_with('}'))
                        || (s.starts_with('[') && s.ends_with(']'))
                    {
                        if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                            walk(&parsed, out, seen, title_hint, artist_hint);
                        }
                    }
                }
                _ => {}
            }
        }

        // 4. Execution
        let mut queries = Vec::new();
        let mut seen = HashSet::new();

        // Start from the most likely branch.
        walk(
            state_data.get("entity").unwrap_or(state_data),
            &mut queries,
            &mut seen,
            None,
            None,
        );

        // Fallback: walk the whole state blob too.
        walk(state_data, &mut queries, &mut seen, None, None);

        if queries.is_empty() {
            println!(
                "Diagnostic: state.data keys = {:?}",
                state_data.as_object().map(|o| o.keys().collect::<Vec<_>>())
            );

            if let Some(entity) = state_data.get("entity") {
                println!(
                    "Diagnostic: entity keys = {:?}",
                    entity.as_object().map(|o| o.keys().collect::<Vec<_>>())
                );
            }

            return Err(
                "Spotify page loaded, but no track objects were found in the HTML JSON.".into(),
            );
        }

        println!("✅ Successfully scraped {} tracks", queries.len());
        Ok(queries)
    }

    async fn playlist_extractor(&self, url: &str) -> Result<Vec<String>, Error> {
        if url.contains("open.spotify.com/playlist/") || url.contains("open.spotify.com/album/") {
            return self.spotify_playlist_extractor(url).await;
        }

        let output = Command::new("yt-dlp")
            .args([
                "--flat-playlist",
                "--yes-playlist",
                "--ignore-config",
                "--no-warnings",
                "--print",
                "webpage_url",
                url,
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let urls: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| line.starts_with("http") && !line.is_empty())
            .collect();

        Ok(urls)
    }
}

#[async_trait]
impl ProviderUtils for Utils<'_> {
    async fn youtube_provider(&self, song: String) -> Result<(Track, SongMetadata), Error> {
        let original_url = song.clone();

        // We determine the search query and the branding (Provider) upfront.
        let (query, do_search, provider) = if song.starts_with("spotifysearch:") {
            // Case A: Result from our Spotify Playlist Scraper
            (song.replace("spotifysearch:", ""), true, Provider::Spotify)
        } else if song.contains("spotify.com/") {
            // Case B: Single Spotify Link (use your existing HTML scraper)
            (
                self.spotify_extractor(&song).await?,
                true,
                Provider::Spotify,
            )
        } else {
            // Case C: Standard YouTube/SoundCloud/Search
            let needs_search = !song.starts_with("http");
            let prov = if needs_search {
                Provider::Unknown
            } else {
                Provider::from_url(&song)
            };
            (song, needs_search, prov)
        };

        let http_client = self.ctx.data().http.clone();

        let src_input = if do_search {
            YoutubeDl::new_search(http_client, query)
        } else {
            YoutubeDl::new(http_client, query)
        };

        let mut src: Input = src_input.into();

        let mut meta = src.aux_metadata().await?;
        let author = self.ctx.author();

        let final_url = if original_url.starts_with("spotifysearch:") {
            // This is a fake query → use resolved YouTube URL
            meta.source_url
                .take()
                .unwrap_or_else(|| "https://youtube.com".to_string())
        } else if provider == Provider::Spotify {
            // Real Spotify link
            original_url
        } else {
            // Normal case (YouTube, SoundCloud, etc.)
            meta.source_url
                .take()
                .unwrap_or_else(|| "https://youtube.com".to_string())
        };

        let provider_logo = match provider {
            Provider::Unknown => Provider::from_url(&final_url),
            _ => provider,
        };

        let info = SongMetadata {
            title: meta.title.take().unwrap_or_else(|| "Unknown".to_string()),
            url: final_url,
            thumbnail: meta.thumbnail.take().unwrap_or_default(),
            duration: meta.duration,

            request_by: author.name.clone(),
            request_by_avatar: author
                .avatar_url()
                .unwrap_or_else(|| author.default_avatar_url()),

            author: meta
                .channel
                .take()
                .unwrap_or_else(|| "Unknown Author".to_string()),

            provider_logo_url: self
                .ctx
                .data()
                .config
                .interactions_registry
                .get_logo(provider_logo),
        };

        let track = Track::new_with_data(src.into(), Arc::new(info.clone()));

        Ok((track, info))
    }
}

impl Utils<'_> {
    pub async fn play(&self, song: String) -> Result<Vec<(Track, SongMetadata)>, Error> {
        let is_playlist =
            song.contains("list=") || song.contains("/sets/") || song.contains("/playlist/");

        let tracks_data: Vec<(Track, SongMetadata)> = if is_playlist {
            let urls = self.playlist_extractor(&song).await?;

            let mut tracks_data = Vec::new();

            for url in urls {
                if url.contains("/sets/") || url.contains("list=") {
                    continue;
                }

                if let Ok(track_data) = self.youtube_provider(url).await {
                    tracks_data.push(track_data);
                }
            }

            // fallback to single if there is nothing
            if tracks_data.is_empty() {
                vec![self.youtube_provider(song).await?]
            } else {
                tracks_data
            }
        } else {
            vec![self.youtube_provider(song).await?]
        };

        Ok(tracks_data)
    }

    pub async fn start_loading_react(&self) -> Result<(), Error> {
        self.add_reactions(&['🔃']).await?;
        Ok(())
    }

    pub async fn end_loading_react(&self) -> Result<(), Error> {
        self.delete_self_reactions(&['🔃']).await?;
        self.add_reactions(&['✅']).await?;
        Ok(())
    }
}
