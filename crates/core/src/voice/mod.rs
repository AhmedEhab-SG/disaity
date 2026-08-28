mod idle;
mod playing;

use serenity::all::{Cache, ChannelId, GuildId, Http};
use songbird::{Call, Songbird};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use idle::IdleEvent;
use playing::PlayingEvent;

#[derive(Clone)]
pub struct VoiceEventCtx {
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
    pub text_channel_id: ChannelId,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub manager: Arc<Songbird>,
}

pub trait RegisterVoiceEvent {
    async fn register(call_lock: &mut Call, cx: &VoiceEventCtx);
}

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

impl VoiceEventCtx {
    pub async fn register_all(&self, call_lock: &mut Call) {
        // Clear default or old handlers to prevent duplicates
        call_lock.remove_all_global_events();

        PlayingEvent::register(call_lock, self).await;

        IdleEvent::register(call_lock, self).await;
    }
}
