use crate::core::voice::SongMetadata;
use serenity::{
    all::{
        ChannelId, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Http,
        MessageId, Timestamp,
    },
    async_trait,
};
use songbird::{Call, Event, EventContext, EventHandler, TrackEvent};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

struct TrackStartNotifier {
    call: Arc<Mutex<Call>>,
    channel_id: ChannelId,
    http: Arc<Http>,
    message_id: Arc<Mutex<Option<MessageId>>>,
}

struct TrackEndNotifier {
    message_id: Arc<Mutex<Option<MessageId>>>,
    channel_id: ChannelId,
    http: Arc<Http>,
}

fn format_duration(d: Option<std::time::Duration>) -> String {
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

#[async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            let message_id = self.message_id.clone();
            let http = self.http.clone();
            let channel_id = self.channel_id;

            tokio::spawn(async move {
                let mut message_id_lock = message_id.lock().await;

                if let Some(old_id) = message_id_lock.take() {
                    channel_id.delete_message(&http, old_id).await.ok();
                }
            });
        }
        None
    }
}

#[async_trait]
impl EventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_listen) = ctx {
            // We only care about the first track starting in this event
            if let Some((_, handle)) = track_listen.first() {
                // Clone everything we need to move into the background task
                let data = handle.data::<SongMetadata>().clone();
                let call = self.call.clone();
                let http = self.http.clone();
                let channel_id = self.channel_id;
                let message_id = self.message_id.clone();

                // Spawning a task prevents the deadlock!
                tokio::spawn(async move {
                    // 1. Lock the call once to get all queue info
                    let (queue_len, total_duration) = {
                        let call_lock = call.lock().await;
                        let queue = call_lock.queue();
                        let current_queue = queue.current_queue();

                        let len = current_queue.len();
                        let total: Duration = current_queue
                            .iter()
                            .filter_map(|h| Some(h.data::<SongMetadata>()))
                            .filter_map(|d| d.duration)
                            .sum();

                        (len, total)
                    };

                    // 2. Build the embed
                    let embed = CreateEmbed::new()
                        .title(&data.title)
                        .url(&data.url)
                        .color(0x0099ff)
                        .field(
                            "Duration",
                            format!("`{}`", format_duration(data.duration)),
                            true,
                        )
                        .thumbnail(&data.thumbnail)
                        .field(
                            "Tracks",
                            format!(
                                "`{} for {}`",
                                queue_len,
                                format_duration(Some(total_duration))
                            ),
                            true,
                        )
                        .author(
                            CreateEmbedAuthor::new(&data.author).icon_url(&data.provider_logo_url),
                        )
                        .footer(
                            CreateEmbedFooter::new(format!("Requested by {}", data.request_by))
                                .icon_url(&data.request_by_avatar),
                        )
                        .timestamp(Timestamp::now());

                    let mut message_id_lock = message_id.lock().await;

                    channel_id
                        .send_message(&http, CreateMessage::new().embed(embed))
                        .await
                        .map(|msg| *message_id_lock = Some(msg.id))
                        .map_err(|e| {
                            *message_id_lock = None;
                            e
                        })
                        .ok();
                });
            }
        }

        None
    }
}

pub async fn register_playing_info(
    call_lock: &mut Call,
    call: Arc<Mutex<Call>>,
    text_channel_id: ChannelId,
    http: Arc<Http>,
) {
    let message_id = Arc::new(Mutex::new(None));

    call_lock.add_global_event(
        Event::Track(TrackEvent::Play),
        TrackStartNotifier {
            call,
            http: http.clone(),
            channel_id: text_channel_id,
            message_id: message_id.clone(),
        },
    );

    call_lock.add_global_event(
        Event::Track(TrackEvent::End),
        TrackEndNotifier {
            http,
            message_id,
            channel_id: text_channel_id,
        },
    );
}
