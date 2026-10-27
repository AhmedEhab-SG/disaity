use poise::command;
use songbird::tracks::PlayMode;

use disaity_core::{Context, ContextExt, Error, say};

use super::super::{Check, Utils};

/// specific time on the current playing track if its passes it
/// it skips the current track HH:MM:SS
#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | SPEAK | ADD_REACTIONS",
    check = "Check::same_vc",
    check = "Check::not_empty_queue",
    check = "Check::not_mute",
    check = "Check::user_not_deafen"
)]
pub async fn seek(
    ctx: Context<'_>,
    #[description = "Enter time number or format you want to play in current song."] time: String,
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

    let track_handler = call_lock
        .queue()
        .current()
        .ok_or("Nothing is in the queue to seek.")?;

    let info = track_handler.get_info().await?;

    if info.playing != PlayMode::Play {
        return Err("The music is must be already playing to seek a time frame in it".into());
    }

    let target = Utils::parse_timestamp(&time).map_err(|msg| {
        format!(
            "Invalid time: {}. Examples: `1:20`, `90`, `1m20s`, `1:02:30`",
            msg
        )
    })?;

    let result_time = track_handler.seek(target).result_async().await?;

    ctx_utils.end_loading_react().await?;

    say!(
        ctx,
        format!("Seeked to {}", Utils::format_duration_human(result_time))
    );

    Ok(())
}
