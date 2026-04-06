use poise::command;

use crate::{
    commands::checks::{not_deafen, not_empty_queue, same_vc},
    core::{context::Context, error::Error},
};

#[command(
    slash_command,
    prefix_command,
    broadcast_typing,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | MANAGE_MESSAGES",
    check = "same_vc",
    check = "not_empty_queue",
    check = "not_deafen"
)]
pub async fn jump(
    ctx: Context<'_>,
    #[description = "Enter song number."] order: usize,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("this commad ont works in servers.").await?;
        return Ok(());
    };

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(call) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    if let Context::Prefix(p_ctx) = ctx {
        p_ctx.msg.react(ctx, '🔄').await?;
    }

    let call_lock = call.lock().await;

    let queue = call_lock.queue();

    if order == 0 {
        ctx.say("Track numbers start from 1.").await?;
        return Ok(());
    }

    let index = order - 1;

    if index >= queue.len() {
        ctx.say("That track number does not exist in the queue.")
            .await?;
        return Ok(());
    }

    if index != 0 {
        queue.modify_queue(|vq| {
            (0..index).for_each(|_| {
                if let Some(queued) = vq.pop_front() {
                    queued.stop().ok();
                }
            });
        });
    } else {
        queue.skip()?;
    }

    if let Context::Prefix(p_ctx) = ctx {
        p_ctx.msg.delete_reactions(ctx).await?;
        p_ctx.msg.react(ctx, '✅').await?;
    }

    ctx.say(format!("▶️ Jumped to track {}!", order)).await?;

    Ok(())
}
