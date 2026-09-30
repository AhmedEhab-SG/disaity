use std::sync::Arc;

use serenity::async_trait;
use songbird::Call;
use tokio::sync::Mutex;

use crate::{errors::Error, voice::VoiceEventCtx};

use super::Utils;

#[async_trait]
pub trait VoiceUtils {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error>;
}

// TODO move out VoiceUitls to voice mod
#[async_trait]
impl VoiceUtils for Utils<'_> {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error> {
        let serenity_context = self.ctx.serenity_context();
        let guild_id = self
            .ctx
            .guild_id()
            .ok_or("This command only works in servers.")?;
        let voice_channel_id = self
            .ctx
            .guild()
            .and_then(|g| {
                g.voice_states
                    .get(&self.ctx.author().id)
                    .and_then(|vs| vs.channel_id)
            })
            .ok_or("You must be in a voice channel!")?;
        let manager = songbird::get(serenity_context)
            .await
            .ok_or("Failed to mount songbird")?;

        let (call, is_new_call) = if let Some(exisiting_call) = manager.get(guild_id) {
            let mut call_lock = exisiting_call.lock().await;
            call_lock.join(voice_channel_id).await.ok();
            drop(call_lock);

            (exisiting_call, false)
        } else {
            let new_call = manager.join(guild_id, voice_channel_id).await?;
            (new_call, true)
        };

        if is_new_call {
            let cx = VoiceEventCtx {
                call: call.clone(),
                guild_id,
                manager,
                text_channel_id: self.ctx.channel_id(),
                http: serenity_context.http.clone(),
                cache: serenity_context.cache.clone(),
            };

            let mut call_lock = call.lock().await;
            cx.register_all(&mut call_lock).await;
        }

        Ok(call)
    }
}
