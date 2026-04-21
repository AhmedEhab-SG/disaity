use poise::command;

use crate::{
    commands::checks::{not_empty_queue, same_vc, user_not_deafen},
    core::{
        context::{Context, ContextExt},
        errors::Error,
        macros::say,
    },
};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "same_vc",
    check = "not_empty_queue",
    check = "user_not_deafen"
)]
pub async fn jump(
    ctx: Context<'_>,
    #[description = "Enter song number."] order: usize,
) -> Result<(), Error> {
    let ctx_utils = ctx.utils();
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager.get(guild_id).ok_or("Not in a voice channel!")?;

    ctx_utils.start_loading_react().await?;

    let call_lock = call.lock().await;

    let queue = call_lock.queue();

    if order == 0 {
        return Err("Track numbers start from 1.".into());
    }

    let index = order - 1;

    if index >= queue.len() {
        return Err("That track number does not exist in the queue.".into());
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

    ctx.utils().end_loading_react().await?;

    say!(ctx, format!("▶️ Jumped to track {}!", order));

    Ok(())
}
