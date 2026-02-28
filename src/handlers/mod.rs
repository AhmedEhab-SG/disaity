pub mod idle;
pub mod playing;

use serenity::all::{ChannelId, GuildId, Http};
use songbird::{Call, Songbird};
use std::sync::Arc;

use crate::handlers::{idle::register_idle_timeout, playing::register_playing_info};

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: Option<std::time::Duration>,
    pub request_by: String,
}

pub async fn register_all(
    call: &mut Call,
    guild_id: GuildId,
    text_channel_id: ChannelId,
    http: Arc<Http>,
    manager: Arc<Songbird>,
) {
    // Clear default or old handlers to prevent duplicates
    call.remove_all_global_events();

    register_playing_info(call, text_channel_id, http).await;

    register_idle_timeout(call, guild_id, manager).await;
}
