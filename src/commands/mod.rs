pub mod music;
pub mod others;

use crate::{core::Error, handlers::register_all};
use serenity::all::{Cache, ChannelId};
use songbird::Call;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn get_or_join_voice(
    manager: &Arc<songbird::Songbird>,
    guild_id: serenity::all::GuildId,
    voice_channel_id: ChannelId,
    text_channel_id: ChannelId,
    http: Arc<serenity::all::Http>,
    cache: Arc<Cache>,
) -> Result<Arc<Mutex<Call>>, Error> {
    let (call, is_new_call) = if let Some(exisiting_call) = manager.get(guild_id) {
        let mut handler = exisiting_call.lock().await;

        handler.join(voice_channel_id).await.ok();

        drop(handler);

        (exisiting_call.clone(), false)
    } else {
        let new_call = manager.join(guild_id, voice_channel_id).await?;
        (new_call, true)
    };

    if is_new_call {
        let mut call_lock = call.lock().await;

        register_all(
            &mut call_lock,
            guild_id,
            text_channel_id,
            http,
            manager.clone(),
            cache,
        )
        .await;
    }

    Ok(call)
}
