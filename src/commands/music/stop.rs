use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let serenity_context = ctx.serenity_context();
    let cache = &serenity_context.cache;

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("this commad ont works in servers.").await?;
        return Ok(());
    };

    let Some(user_ch) = cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("you must be in a voice channel").await?;
        return Ok(());
    };

    let Some(client_ch) = cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("I'm not in any channel").await?;
        return Ok(());
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let Some(manager) = songbird::get(serenity_context).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        ctx.say("Nothing is currently in queue to stop.").await?;
        return Ok(());
    };

    queue.modify_queue(|q| {
        q.clear();
    });

    handler.stop();

    ctx.say("⏹ Stopped playback!").await?;

    Ok(())
}
