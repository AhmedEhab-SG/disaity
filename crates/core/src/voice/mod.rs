mod idle;
mod playing;

use serenity::all::{Cache, ChannelId, GuildId, Http};
use songbird::{Call, Songbird};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::voice::{idle::register_idle_timeout, playing::register_playing_info};

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: Option<Duration>,
    pub request_by: String,
    pub request_by_avatar: String,
    pub author: String,
    pub provider_logo_url: String,
}

pub async fn register_all(
    call_lock: &mut Call,
    call: Arc<Mutex<Call>>,
    guild_id: GuildId,
    text_channel_id: ChannelId,
    http: Arc<Http>,
    manager: Arc<Songbird>,
    cache: Arc<Cache>,
) {
    // Clear default or old handlers to prevent duplicates
    call_lock.remove_all_global_events();

    register_playing_info(call_lock, call, text_channel_id, http).await;

    register_idle_timeout(call_lock, guild_id, manager, cache).await;
}
