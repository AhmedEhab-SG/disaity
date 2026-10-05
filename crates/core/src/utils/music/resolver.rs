use std::{sync::Arc, time::Duration};

use reqwest::Client;
use songbird::{
    input::{Input, YoutubeDl},
    tracks::Track,
};

use disaity_config::{Assets, Provider};

use crate::{binaries::Binaries, context::Context, errors::Error};

use super::{TrackMetadata, playlist::ListedMeta, spotify::Spotify};

pub(super) type ResolvedTrack = (Track, TrackMetadata);

/// Turns a song query into a playable track. Unlike `Utils`, it owns
/// everything it needs, so it can keep resolving songs in a background task
/// after the command has replied.
pub(super) struct TrackResolver {
    pub(super) http: Client,
    assets: Assets,
    request_by: String,
    request_by_avatar: String,
}

impl TrackResolver {
    pub(super) fn new(ctx: Context<'_>) -> Self {
        let author = ctx.author();

        Self {
            http: ctx.data().http.clone(),
            assets: ctx.data().config.assets.clone(),
            request_by: author.name.clone(),
            request_by_avatar: author
                .avatar_url()
                .unwrap_or_else(|| author.default_avatar_url()),
        }
    }

    /// Resolves a URL, a search, a Spotify link, or a query from the Spotify
    /// playlist scraper. Runs yt-dlp once to read the song's metadata.
    pub(super) async fn resolve(&self, song: String) -> Result<ResolvedTrack, Error> {
        if let Some(query) = song.strip_prefix(Spotify::SEARCH_PREFIX) {
            return self
                .fetch(query.to_string(), true, Provider::Spotify, None)
                .await;
        }

        if song.contains("spotify.com/") {
            // Spotify can't be streamed: read the title from its page and search it.
            let title = Spotify::song_title(&self.http, &song).await?;
            return self.fetch(title, true, Provider::Spotify, Some(song)).await;
        }

        let search = !song.starts_with("http");
        let provider = if search {
            Provider::Unknown
        } else {
            Provider::from_url(&song)
        };

        self.fetch(song, search, provider, None).await
    }

    /// `link` overrides the URL shown in the embed, for a Spotify link that was
    /// searched on YouTube.
    async fn fetch(
        &self,
        query: String,
        search: bool,
        provider: Provider,
        link: Option<String>,
    ) -> Result<ResolvedTrack, Error> {
        let mut src: Input = self.source(query, search)?.into();
        let mut meta = src.aux_metadata().await?;

        let url = link
            .or_else(|| meta.source_url.take())
            .unwrap_or_else(|| "https://youtube.com".to_string());

        let provider = match provider {
            Provider::Unknown => Provider::from_url(&url),
            provider => provider,
        };

        let info = self.track_metadata(
            meta.title.take().unwrap_or_else(|| "Unknown".to_string()),
            url,
            meta.thumbnail.take().unwrap_or_default(),
            meta.duration,
            meta.channel
                .take()
                .unwrap_or_else(|| "Unknown Author".to_string()),
            provider,
        );

        Ok((Track::new_with_data(src, Arc::new(info.clone())), info))
    }

    /// A track for a song whose metadata came with the playlist listing.
    /// Nothing runs yet: yt-dlp only starts when the song is about to play.
    pub(super) fn listed(&self, url: String, meta: ListedMeta) -> Result<ResolvedTrack, Error> {
        let info = self.track_metadata(
            meta.title,
            url.clone(),
            meta.thumbnail.unwrap_or_default(),
            meta.duration,
            meta.author.unwrap_or_else(|| "Unknown Author".to_string()),
            Provider::from_url(&url),
        );

        let src: Input = self.source(url, false)?.into();

        Ok((Track::new_with_data(src, Arc::new(info.clone())), info))
    }

    fn source(&self, query: String, search: bool) -> Result<YoutubeDl<'static>, Error> {
        let binaries = Binaries::get()?;
        let ytdlp = binaries.ytdlp.program();

        let src = if search {
            YoutubeDl::new_search_ytdl_like(ytdlp, self.http.clone(), query)
        } else {
            YoutubeDl::new_ytdl_like(ytdlp, self.http.clone(), query)
        };

        Ok(src.user_args(binaries.ffmpeg_args()))
    }

    fn track_metadata(
        &self,
        title: String,
        url: String,
        thumbnail: String,
        duration: Option<Duration>,
        author: String,
        provider: Provider,
    ) -> TrackMetadata {
        TrackMetadata {
            title,
            url,
            thumbnail,
            duration,
            author,
            request_by: self.request_by.clone(),
            request_by_avatar: self.request_by_avatar.clone(),
            provider_logo_url: self.assets.get_logo(provider),
        }
    }
}
