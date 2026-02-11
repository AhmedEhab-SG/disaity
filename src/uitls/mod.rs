use serenity::all::ChannelId;
use songbird::{Call, Event, TrackEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{core::Error, handlers::playing::TrackStartNotifier};

pub async fn get_or_join_voice(
    manager: &Arc<songbird::Songbird>,
    guild_id: serenity::all::GuildId,
    voice_channel_id: ChannelId,
    text_channel_id: ChannelId,
    http: Arc<serenity::all::Http>,
) -> Result<Arc<Mutex<Call>>, Error> {
    let call = manager.join(guild_id, voice_channel_id).await?;

    let mut handler = call.lock().await;

    // handle that later
    handler.remove_all_global_events();

    handler.add_global_event(
        Event::Track(TrackEvent::Play),
        TrackStartNotifier {
            channel_id: text_channel_id,
            http: http.clone(),
        },
    );

    drop(handler);

    Ok(call)
}
