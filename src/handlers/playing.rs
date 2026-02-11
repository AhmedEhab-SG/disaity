use crate::handlers::SongMetadata;
use serenity::{
    all::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Http},
    async_trait,
};
use songbird::{Event, EventContext, EventHandler};
use std::sync::Arc;

pub struct TrackStartNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
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
impl EventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_listen) = ctx {
            for (_, handle) in track_listen.iter() {
                let data = handle.data::<SongMetadata>();

                let embed = CreateEmbed::new()
                    .title(data.title.clone())
                    .url(data.url.clone())
                    .thumbnail(data.thumbnail.clone())
                    .color(0xFF0000)
                    .field("Duration", format_duration(data.duration), true)
                    .footer(CreateEmbedFooter::new(format!(
                        "Requested by {}",
                        data.request_by
                    )));

                self.channel_id
                    .send_message(&self.http, CreateMessage::new().embed(embed))
                    .await
                    .ok();
            }
        }

        None
    }
}
