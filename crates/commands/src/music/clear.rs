use poise::command;

use crate::checks::{not_empty_queue, same_vc};
use disaity_core::{Context, ContextExt, Error, say};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "same_vc",
    check = "not_empty_queue"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
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

    queue.modify_queue(|q| {
        q.clear();
    });

    ctx_utils.end_loading_react().await?;

    say!(ctx, "🗑️ Queue cleared!", application_only);

    Ok(())
}
