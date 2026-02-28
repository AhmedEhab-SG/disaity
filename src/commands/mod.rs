pub mod music;
pub mod others;

use crate::{core::Error, handlers::register_all};
use serenity::all::ChannelId;
use songbird::Call;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn get_or_join_voice(
    manager: &Arc<songbird::Songbird>,
    guild_id: serenity::all::GuildId,
    voice_channel_id: ChannelId,
    text_channel_id: ChannelId,
    http: Arc<serenity::all::Http>,
) -> Result<Arc<Mutex<Call>>, Error> {
    let call = manager.join(guild_id, voice_channel_id).await?;

    let mut call_lock = call.lock().await;

    // handle that later
    register_all(
        &mut call_lock,
        guild_id,
        text_channel_id,
        http,
        manager.clone(),
    )
    .await;

    drop(call_lock);

    Ok(call)
}
