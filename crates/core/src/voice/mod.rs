mod idle;
mod playing;

use std::sync::Arc;

use serenity::all::{Cache, ChannelId, GuildId, Http};
use songbird::{Call, Songbird};
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

impl VoiceEventCtx {
    pub async fn register_all(&self, call_lock: &mut Call) {
        // Clear default or old handlers to prevent duplicates
        call_lock.remove_all_global_events();

        PlayingEvent::register(call_lock, self).await;

        IdleEvent::register(call_lock, self).await;
    }
}
