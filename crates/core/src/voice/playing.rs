use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use serenity::{
    all::{
        ChannelId, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Http,
        MessageId, Timestamp,
    },
    async_trait,
};
use songbird::{
    Call, Event, EventContext, EventHandler, TrackEvent,
    tracks::{TrackHandle, TrackQueue},
};

use super::{super::TrackMetadata, RegisterVoiceEvent, VoiceEventCtx};

/// event to describes the current playing track

struct OnPlay(NowPlaying);
struct OnEnd(NowPlaying);

#[async_trait]
impl EventHandler for OnPlay {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(tracks) = ctx else {
            return None;
        };

        if let Some((_, track)) = tracks.first() {
            self.0.show(track).await;
        }

        None
    }
}

#[async_trait]
impl EventHandler for OnEnd {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(tracks) = ctx else {
            return None;
        };

        if let Some((_, track)) = tracks.first() {
            self.0.clear(track).await;
        }

        None
    }
}

#[derive(Clone)]
struct NowPlaying {
    http: Arc<Http>,
    channel_id: ChannelId,
    queue: TrackQueue,
    current: Arc<Mutex<Option<(TrackHandle, MessageId)>>>,
}

impl NowPlaying {
    async fn show(&self, track: &TrackHandle) {
        let previous = {
            let mut current = self.current.lock().unwrap();

            // Same track = resumed after pause, keep the existing message
            if current
                .as_ref()
                .is_some_and(|(t, _)| t.uuid() == track.uuid())
            {
                return;
            }

            current.take()
        };

        if let Some((_, old)) = previous {
            self.channel_id.delete_message(&self.http, old).await.ok();
        }

        let message = CreateMessage::new().embed(self.embed(track));

        if let Ok(msg) = self.channel_id.send_message(&self.http, message).await {
            *self.current.lock().unwrap() = Some((track.clone(), msg.id));
        }
    }

    async fn clear(&self, track: &TrackHandle) {
        // Only delete if the message belongs to the track that ended
        let previous = self
            .current
            .lock()
            .unwrap()
            .take_if(|(t, _)| t.uuid() == track.uuid());

        if let Some((_, old)) = previous {
            self.channel_id.delete_message(&self.http, old).await.ok();
        }
    }

    fn embed(&self, track: &TrackHandle) -> CreateEmbed {
        let data = track.data::<TrackMetadata>();
        let queue = self.queue.current_queue();

        let total: Duration = queue
            .iter()
            .filter_map(|h| h.data::<TrackMetadata>().duration)
            .sum();

        CreateEmbed::new()
            .title(&data.title)
            .url(&data.url)
            .color(0x0099ff)
            .field(
                "Duration",
                format!("`{}`", self.format_duration(data.duration)),
                true,
            )
            .thumbnail(&data.thumbnail)
            .field(
                "Tracks",
                format!(
                    "`{} for {}`",
                    queue.len(),
                    self.format_duration(Some(total))
                ),
                true,
            )
            .author(CreateEmbedAuthor::new(&data.author).icon_url(&data.provider_logo_url))
            .footer(
                CreateEmbedFooter::new(format!("Requested by {}", data.request_by))
                    .icon_url(&data.request_by_avatar),
            )
            .timestamp(Timestamp::now())
    }

    fn format_duration(&self, d: Option<std::time::Duration>) -> String {
        match d {
            Some(d) => {
                let seconds = d.as_secs();
                let minutes = seconds / 60;
                let rem_seconds = seconds % 60;
                format!("{:02}:{:02}", minutes, rem_seconds)
            }
            None => "Live/Unknown".to_string(),
        }
    }
}

pub(super) struct PlayingEvent;

impl RegisterVoiceEvent for PlayingEvent {
    async fn register(call: &mut Call, cx: &VoiceEventCtx) {
        let now_playing = NowPlaying {
            http: cx.http.clone(),
            channel_id: cx.text_channel_id,
            queue: call.queue().clone(),
            current: Arc::default(),
        };

        call.add_global_event(Event::Track(TrackEvent::Play), OnPlay(now_playing.clone()));
        call.add_global_event(Event::Track(TrackEvent::End), OnEnd(now_playing));
    }
}
