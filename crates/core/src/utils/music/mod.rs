mod playlist;
mod resolver;
mod spotify;

use std::{sync::Arc, time::Duration};

use songbird::{Call, tracks::Track};
use tokio::sync::Mutex;

use crate::errors::Error;

use super::Utils;

use resolver::{ResolvedTrack, TrackResolver};

/// What `Utils::play` put in the queue. The rest of a playlist may still be
/// resolving in the background.
pub struct Queued {
    pub first_title: String,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: Option<Duration>,
    pub request_by: String,
    pub request_by_avatar: String,
    pub author: String,
    pub provider_logo_url: String,
}

/// Songs of one request: `ready` can be queued now, `pending` still needs its
/// own yt-dlp lookup.
#[derive(Default)]
struct Batch {
    ready: Vec<ResolvedTrack>,
    pending: Vec<String>,
}

impl Utils<'_> {
    /// Queues `song` and returns as soon as something can play. For playlists,
    /// songs that still need a lookup keep loading in the background.
    pub async fn play(&self, song: String, call: Arc<Mutex<Call>>) -> Result<Queued, Error> {
        let resolver = TrackResolver::new(self.ctx);

        let mut batch = if playlist::is_playlist(&song) {
            playlist::load(&resolver, &song).await?
        } else {
            Batch::default()
        };

        // Not a playlist, or nothing in it could be loaded: play it as one song.
        if batch.ready.is_empty() {
            batch.ready.push(resolver.resolve(song).await?);
        }

        let queued = Queued {
            first_title: batch.ready[0].1.title.clone(),
            total: batch.ready.len() + batch.pending.len(),
        };

        let mut call_lock = call.lock().await;
        for (track, info) in batch.ready {
            enqueue(&mut call_lock, track, &info);
        }
        drop(call_lock);

        if !batch.pending.is_empty() {
            tokio::spawn(enqueue_in_background(resolver, batch.pending, call));
        }

        Ok(queued)
    }
}

/// Queues a track, preloading the next one 5s before this one ends.
/// `Call::enqueue` would run yt-dlp again just to read the duration we already
/// have, while holding the call lock.
fn enqueue(call_lock: &mut Call, track: Track, info: &TrackMetadata) {
    let preload = info
        .duration
        .map(|d| d.saturating_sub(Duration::from_secs(5)));
    call_lock.enqueue_with_preload(track, preload);
}

/// Resolves the rest of a playlist one song at a time, queueing each as soon as
/// it is ready.
async fn enqueue_in_background(
    resolver: TrackResolver,
    queries: Vec<String>,
    call: Arc<Mutex<Call>>,
) {
    for query in queries {
        let Ok((track, info)) = resolver.resolve(query).await else {
            continue;
        };

        let mut call_lock = call.lock().await;

        // The bot left, or /stop or /clear emptied the queue: drop the rest.
        if call_lock.current_channel().is_none() || call_lock.queue().is_empty() {
            break;
        }

        enqueue(&mut call_lock, track, &info);
    }
}
