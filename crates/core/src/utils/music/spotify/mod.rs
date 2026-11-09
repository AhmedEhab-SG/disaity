mod walk;

use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::errors::Error;

use walk::Walker;

pub(super) struct Spotify;

impl Spotify {
    /// Marks a playlist song that has to be searched by title and artist, since
    /// Spotify itself can't be streamed.
    const URL: &str = "open.spotify.com/";
    pub(super) const SEARCH_PREFIX: &str = "spotifysearch:";
    const LIST_INDICATORS: [&str; 2] = ["playlist/", "album/"];
    const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    pub(super) fn is_list(url: &str) -> bool {
        url.split_once(Self::URL)
            .is_some_and(|(_, path)| Self::LIST_INDICATORS.iter().any(|p| path.contains(p)))
    }

    /// The "title artist" text of a single Spotify song, read from its meta
    /// tags. The `<title>` wording changes between songs, these don't.
    ///
    /// No browser User-Agent here: with one, Spotify serves the web player
    /// shell instead of the song page.
    pub(super) async fn song_title(http: &Client, url: &str) -> Result<String, Error> {
        let html = http.get(url).send().await?.text().await?;
        let document = Html::parse_document(&html);

        let meta = |selector: &str| {
            let selector = Selector::parse(selector).ok()?;
            document
                .select(&selector)
                .next()?
                .value()
                .attr("content")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };

        let title =
            meta(r#"meta[property="og:title"]"#).ok_or("failed to extract the song title")?;
        let artist = meta(r#"meta[name="music:musician_description"]"#).unwrap_or_default();

        Ok(format!("{title} {artist}").trim_end().to_string())
    }

    /// A search query per track of a Spotify playlist or album, scraped from the
    /// JSON the page embeds.
    pub(super) async fn playlist_queries(http: &Client, url: &str) -> Result<Vec<String>, Error> {
        let html = Self::fetch_page(http, url).await?;

        let state = Self::page_state(&html)?;

        let queries = Self::track_queries(&state);

        if queries.is_empty() {
            tracing::debug!(
                "Diagnostic: state.data keys = {:?}",
                state.as_object().map(|o| o.keys().collect::<Vec<_>>())
            );

            if let Some(entity) = state.get("entity") {
                tracing::debug!(
                    "Diagnostic: entity keys = {:?}",
                    entity.as_object().map(|o| o.keys().collect::<Vec<_>>())
                );
            }

            return Err(
                "Spotify page loaded, but no track objects were found in the HTML JSON.".into(),
            );
        }

        tracing::debug!("✅ Successfully scraped {} tracks", queries.len());
        Ok(queries)
    }

    /// A search query per track found anywhere in Spotify's page state. The page
    /// layout changes often, so this looks for track-shaped objects instead of
    /// following one fixed path.
    fn track_queries(state: &Value) -> Vec<String> {
        let mut walker = Walker::default();

        // Start from the most likely branch.
        walker.walk(state.get("entity").unwrap_or(state), None, None);

        // Fallback: walk the whole state blob too.
        walker.walk(state, None, None);

        walker.queries
    }

    /// The embed page is tried first, then the regular one.
    async fn fetch_page(http: &Client, url: &str) -> Result<String, Error> {
        let clean_url = url.split('?').next().unwrap_or(url).trim_end_matches('/');

        let embed_url = if clean_url.contains("/embed/") {
            clean_url.to_string()
        } else {
            clean_url.replace(Self::URL, format!("{}embed/", Self::URL).as_str())
        };

        for candidate in [embed_url.as_str(), clean_url] {
            if let Ok(resp) = http
                .get(candidate)
                .header("User-Agent", Self::USER_AGENT)
                .header("Accept-Language", "en-US,en;q=0.9")
                .send()
                .await
                && let Ok(text) = resp.text().await
                && !text.is_empty()
            {
                return Ok(text);
            }
        }

        Err("Failed to fetch Spotify page".into())
    }

    /// `props.pageProps.state.data` of the page's `__NEXT_DATA__` script.
    fn page_state(html: &str) -> Result<Value, Error> {
        let document = Html::parse_document(html);
        let selector =
            Selector::parse("script#__NEXT_DATA__").map_err(|_| "Invalid CSS selector")?;

        let json_str = document
            .select(&selector)
            .next()
            .map(|element| element.inner_html())
            .ok_or_else(|| {
                // the error itself goes back to the caller, this is only the clue.
                // chars, not bytes: slicing inside a multi-byte char panics
                tracing::debug!(
                    "--- RAW HTML START ---\n{}\n--- RAW HTML END ---",
                    html.chars().take(1000).collect::<String>()
                );
                "Could not find script#__NEXT_DATA__ in Spotify HTML"
            })?;

        let mut root: Value = serde_json::from_str(&json_str)?;

        let state = root
            .pointer_mut("/props/pageProps/state/data")
            .map(Value::take)
            .ok_or("Spotify state.data missing")?;

        Ok(state)
    }
}
