use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "stop", aliases("st"))]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
        return Ok(());
    };

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;

    handler.stop();

    ctx.say("⏹ Stopped playback!").await?;

    Ok(())
}
