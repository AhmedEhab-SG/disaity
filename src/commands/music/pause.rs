use poise::command;
use songbird::tracks::PlayMode;

use crate::{
    commands::checks::{not_empty_queue, same_vc},
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
    check = "not_empty_queue"
)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let ctx_utils = ctx.utils();
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager.get(guild_id).ok_or("Not in a voice channel!")?;

    ctx_utils.start_loading_react().await?;

    let call_lock = call.lock().await;

    let handle = call_lock
        .queue()
        .current()
        .ok_or("Failed to get currrent queue")?;

    let state = handle.get_info().await?;

    if state.playing == PlayMode::Pause {
        return Err("The music is not playing".into());
    }

    handle.pause()?;

    ctx_utils.end_loading_react().await?;

    say!(ctx, "⏸️ paused");

    Ok(())
}
