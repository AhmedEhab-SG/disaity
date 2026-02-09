use serenity::all::ChannelId;
use songbird::{Call, Event, TrackEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::handlers::playing::TrackStartNotifier;

pub async fn get_or_join_voice(
    manager: &Arc<songbird::Songbird>,
    guild_id: serenity::all::GuildId,
    channel_id: ChannelId,
    http: Arc<serenity::all::Http>,
) -> Result<Arc<Mutex<Call>>, String> {
    // 1. Check if we are ALREADY connected
    if let Some(call) = manager.get(guild_id) {
        // If we find a call, it means we joined previously.
        // Since we add the event on join, we KNOW it's already there.
        // We do NOTHING here. Just return the existing call.
        return Ok(call);
    }

    let call = manager
        .join(guild_id, channel_id)
        .await
        .map_err(|e| format!("Failed to join voice: {:?}", e))?;

    // 3. Register the Event (Runs ONLY on the very first join)
    let mut handler = call.lock().await;

    // We do NOT use remove_all_global_events() here.
    // We know this is a fresh connection, so it's clean.
    handler.add_global_event(
        Event::Track(TrackEvent::Play),
        TrackStartNotifier { channel_id, http },
    );

    drop(handler); // Unlock

    Ok(call)
}
