use poise::command;

use crate::{
    commands::checks::{not_empty_queue, same_vc},
    core::{context::Context, error::Error},
};

#[command(
    slash_command,
    prefix_command,
    broadcast_typing,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | MANAGE_MESSAGES",
    check = "same_vc",
    check = "not_empty_queue"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
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

    if let Context::Prefix(p_ctx) = ctx {
        p_ctx.msg.react(ctx, '🔄').await?;
    }

    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    queue.modify_queue(|q| {
        q.clear();
    });

    if let Context::Prefix(p_ctx) = ctx {
        p_ctx.msg.delete_reactions(ctx).await?;
        p_ctx.msg.react(ctx, '✅').await?;
    }

    ctx.say("🗑️ Queue cleared!").await?;
    Ok(())
}
