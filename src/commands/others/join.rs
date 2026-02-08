use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "join", aliases("j"))]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let voice_channel = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    });

    let Some(channel_id) = voice_channel else {
        ctx.say("You are already not in any chennel").await?;
        return Ok(());
    };

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        handler.stop();

        let queue = handler.queue();

        if !queue.is_empty() {
            queue.modify_queue(|q| {
                q.clear();
            });
        }
    }

    manager.join(guild_id, channel_id).await?;

    ctx.say("Yes?").await?;

    Ok(())
}
