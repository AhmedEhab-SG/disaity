use poise::command;

use disaity_core::{Context, ContextExt, Error, say};

use super::super::Check;

/// clears the current music queue
#[command(
    slash_command,
    prefix_command,
    guild_only,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "Check::same_vc",
    check = "Check::not_empty_queue"
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

    // modfiy the queue but leaves the 1st current song
    let removed = queue.modify_queue(|q| {
        if q.len() <= 1 {
            return None;
        }
        Some(q.drain(1..).collect::<Vec<_>>())
    });

    let Some(removed) = removed else {
        return Err("only the current track is queued".into());
    };

    // stops the cleared tracks and avoid leaking
    for track in &removed {
        track.stop().ok();
    }

    ctx_utils.end_loading_react().await?;

    say!(
        ctx,
        format!("🗑️ Queue cleared! total: {}", removed.len()),
        application_only
    );

    Ok(())
}
