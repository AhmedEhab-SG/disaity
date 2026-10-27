use poise::command;

use disaity_core::{Context, ContextExt, Error, say};

use super::super::Check;

/// jumps to a number in the queue (forward only)
#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "Check::same_vc",
    check = "Check::not_empty_queue",
    check = "Check::user_not_deafen"
)]
pub async fn jump(
    ctx: Context<'_>,
    #[description = "Enter song number."] order: u32,
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

    let index = order as usize - 1;

    if index >= queue.len() {
        return Err("That track number does not exist in the queue.".into());
    }

    // remove and stop every track before the target
    queue.modify_queue(|vq| {
        for queued in vq.drain(..index) {
            queued.stop().ok();
        }
    });

    // the target is now at the front but queued tracks wait paused, so play it
    queue.resume()?;

    ctx_utils.end_loading_react().await?;

    say!(ctx, format!("▶️ Jumped to track {}!", order));

    Ok(())
}
