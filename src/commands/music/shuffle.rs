use poise::command;

use rand::{rng, seq::SliceRandom};

use crate::core::{context::Context, error::Error};

#[command(slash_command, prefix_command)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
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

    let Some(call) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let handler = call.lock().await;

    let queue = handler.queue();

    if queue.len() <= 1 {
        ctx.say("Nothing is currently in queue to shuffle.").await?;
        return Ok(());
    };

    queue.modify_queue(|q| {
        let mut rest: Vec<_> = q.drain(1..).collect();

        rest.shuffle(&mut rng());

        q.extend(rest.into_iter());
    });

    ctx.say("Shuffled the queue.").await?;

    Ok(())
}
