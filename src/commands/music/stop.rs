use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "stop", aliases("st"))]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = manager.get(guild_id);

    if let Some(handler_lock) = handler_lock {
        let mut handler = handler_lock.lock().await;

        handler.stop();

        ctx.say("⏹ Stopped playback!").await?;

        return Ok(());
    }

    ctx.say("Not in a voice channel!").await?;

    Ok(())
}
