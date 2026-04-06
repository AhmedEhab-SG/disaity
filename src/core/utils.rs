use serenity::async_trait;

use crate::{
    core::{context::Context, error::Error},
    handlers::register_all,
};

pub struct Utils<'a> {
    pub ctx: Context<'a>,
}

use songbird::{Call, Songbird};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait UtilsExt {
    async fn add_reactions(&self, emojis: &[char]) -> Result<(), Error>;

    async fn get_or_join_voice(&self, manager: Arc<Songbird>) -> Result<Arc<Mutex<Call>>, Error>;
}

#[async_trait]
impl UtilsExt for Utils<'_> {
    async fn add_reactions(&self, _emojis: &[char]) -> Result<(), Error> {
        Ok(())
    }

    async fn get_or_join_voice(&self, manager: Arc<Songbird>) -> Result<Arc<Mutex<Call>>, Error> {
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
        let serenity_context = self.ctx.serenity_context();

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
            let mut call_lock = call.lock().await;

            register_all(
                &mut call_lock,
                guild_id,
                self.ctx.channel_id(),
                serenity_context.http.clone(),
                manager,
                serenity_context.cache.clone(),
            )
            .await;
        }

        Ok(call)
    }
}
