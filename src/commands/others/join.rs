use poise::command;

use crate::core::{
    context::{Context, ContextExt},
    error::Error,
    utils::UtilsExt,
};

#[command(slash_command, prefix_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
        return Ok(());
    };

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
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

    ctx.utils().get_or_join_voice(manager).await?;

    // let mut handler = call.lock().await;

    // handler.deafen(true).await?;

    ctx.say("Yes?").await?;

    Ok(())
}
