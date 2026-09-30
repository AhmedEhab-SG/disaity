mod walk;

use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::errors::Error;

/// Marks a playlist song that has to be searched by title and artist, since
/// Spotify itself can't be streamed.
pub(super) const SEARCH_PREFIX: &str = "spotifysearch:";

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub(super) fn is_playlist(url: &str) -> bool {
    url.contains("open.spotify.com/playlist/") || url.contains("open.spotify.com/album/")
}

/// The "title artist" text of a single Spotify song, read from its page title.
pub(super) async fn song_title(http: &Client, url: &str) -> Result<String, Error> {
    let res = http.get(url).send().await?.text().await?;

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

/// A search query per track of a Spotify playlist or album, scraped from the
/// JSON the page embeds.
pub(super) async fn playlist_queries(http: &Client, url: &str) -> Result<Vec<String>, Error> {
    let html = fetch_page(http, url).await?;
    let state = page_state(&html)?;

    let queries = walk::track_queries(&state);

    if queries.is_empty() {
        println!(
            "Diagnostic: state.data keys = {:?}",
            state.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );

        if let Some(entity) = state.get("entity") {
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

/// The embed page is tried first, then the regular one.
async fn fetch_page(http: &Client, url: &str) -> Result<String, Error> {
    let clean_url = url.split('?').next().unwrap_or(url).trim_end_matches('/');

    let embed_url = if clean_url.contains("/embed/") {
        clean_url.to_string()
    } else {
        clean_url.replace("open.spotify.com/", "open.spotify.com/embed/")
    };

    for candidate in [embed_url.as_str(), clean_url] {
        if let Ok(resp) = http
            .get(candidate)
            .header("User-Agent", USER_AGENT)
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
    let selector = Selector::parse("script#__NEXT_DATA__").map_err(|_| "Invalid CSS selector")?;

    let json_str = document
        .select(&selector)
        .next()
        .map(|element| element.inner_html())
        .ok_or_else(|| {
            println!(
                "--- RAW HTML START ---\n{}\n--- RAW HTML END ---",
                &html[..html.len().min(1000)]
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
