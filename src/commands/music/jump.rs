use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command)]
pub async fn jump(
    ctx: Context<'_>,
    #[description = "Enter song number."] number: usize,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("this commad ont works in servers.").await?;
        return Ok(());
    };

    let Some(user_ch) = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("you must be in a voice channel").await?;
        return Ok(());
    };

    let Some(client_ch) = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("I'm not in any channel").await?;
        return Ok(());
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(call) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let handler = call.lock().await;

    let queue = handler.queue();

    if number == 0 {
        ctx.say("Track numbers start from 1.").await?;
        return Ok(());
    }

    let index = number - 1;

    if index >= queue.len() {
        ctx.say("That track number does not exist in the queue.")
            .await?;
        return Ok(());
    }

    if index == 0 {
        if let Err(_) = queue.skip() {
            ctx.say("Failed to skip the current track.").await?;
        } else {
            ctx.say(format!("▶️ Jumped to track {}!", number)).await?;
        }
        return Ok(());
    }

    queue.modify_queue(|vq| {
        (0..index).for_each(|_| {
            if let Some(queued) = vq.pop_front() {
                queued.stop().ok();
            }
        });
    });

    queue.skip().ok();

    ctx.say(format!("▶️ Jumped to track {}!", number)).await?;

    Ok(())
}
