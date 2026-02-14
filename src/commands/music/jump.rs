use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "jump", aliases("jp"))]
pub async fn jump(ctx: Context<'_>, #[rest] number: usize) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    // let Some(guild_id) = ctx.guild_id() else {
    //     ctx.say("this commad ont works in servers.").await?;
    //     return Ok(());
    // };

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

    let call = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.say("Not in a voice channel!").await?;
            return Ok(());
        }
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
