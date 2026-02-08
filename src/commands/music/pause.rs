use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "pause", aliases("ps"))]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let user_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("you must be in a voice channel").await?;
            return Ok(());
        }
    };

    let client_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("I'm not in any channel").await?;
            return Ok(());
        }
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.say("Not in a voice channel!").await?;
            return Ok(());
        }
    };

    let handler = handler_lock.lock().await;

    if let Some(track_handle) = handler.queue().current() {
        let info = track_handle.get_info().await?;

        if info.playing == songbird::tracks::PlayMode::Pause {
            ctx.say("The music is already paused!").await?;
            return Ok(());
        }

        track_handle.pause()?;
        ctx.say("⏸️ Paused").await?;
        return Ok(());
    };

    ctx.say("Nothing is playing right now").await?;

    Ok(())
}
