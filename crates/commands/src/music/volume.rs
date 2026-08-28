use poise::command;
use songbird::tracks::PlayMode;

use disaity_core::{Context, ContextExt, Error, say};

use crate::checks::{not_empty_queue, not_mute, same_vc, user_not_deafen};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "same_vc",
    check = "not_empty_queue",
    check = "not_mute",
    check = "user_not_deafen"
)]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Enter percentage value from 0 to 200."]
    #[min = 0.0]
    #[max = 200.0]
    percentage: f32,
) -> Result<(), Error> {
    let ctx_utils = ctx.utils();

    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    if !(0.0..=200.0).contains(&percentage) {
        return Err("Please enter a volume percentage between 0 and 200.".into());
    }

    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager.get(guild_id).ok_or("Not in a voice channel!")?;

    ctx_utils.start_loading_react().await?;

    let call_lock = call.lock().await;

    let track_handle = call_lock
        .queue()
        .current()
        .ok_or("Nothing is in the queue to change volume.")?;

    let info = track_handle.get_info().await?;

    if info.playing != PlayMode::Play {
        return Err("The music is must be already playing to change it's volume.".into());
    }

    track_handle.set_volume(percentage / 100.0).ok();

    ctx_utils.end_loading_react().await?;

    say!(ctx, format!("changed the volume to {percentage}%"));

    Ok(())
}
