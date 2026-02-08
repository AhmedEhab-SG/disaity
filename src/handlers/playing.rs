use serenity::{
    all::{ChannelId, CreateEmbed, Http},
    async_trait,
};
use songbird::{Event, EventContext, EventHandler, TrackEvent};
use std::sync::Arc;

use crate::handlers::SongMetadata;

pub struct TrackStartNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
}

#[async_trait]
impl EventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_listen) = ctx {
            for (_, handle) in track_listen.iter() {
                if let Ok(data) = handle.data::<SongMetadata>() {
                    let embed = CreateEmbed::new()
                        .title(data.title.as_deref().unwrap_or("Unknown Title"))
                        .url(
                            metadata
                                .source_url
                                .as_deref()
                                .unwrap_or("https://youtube.com"),
                        )
                        .thumbnail(metadata.thumbnail.as_deref().unwrap_or(""))
                        .color(0xFF0000)
                        .field("Duration", format_duration(metadata.duration), true)
                        .field(
                            "Channel",
                            metadata.channel.as_deref().unwrap_or("Unknown"),
                            true,
                        )
                        .footer(CreateEmbedFooter::new(format!(
                            "Requested by {}",
                            ctx.author().name
                        )));
                }
            }
        }

        None
    }
}
