use poise::{ChoiceParameter, command};
use serenity::all::ReactionType;
use songbird::tracks::LoopState;

use crate::{
    commands::checks::{not_empty_queue, not_mute, same_vc, user_not_deafen},
    core::{
        context::{Context, ContextExt},
        errors::Error,
        macros::say,
        utils::ReactionUtils,
    },
};

#[derive(Debug, ChoiceParameter)]
enum RepeatMode {
    #[name = "song"]
    Song,
    #[name = "toggle"]
    Toggle,
    #[name = "disable"]
    Disable,
}

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
pub async fn repeat(
    ctx: Context<'_>,
    #[description = "Repeat's a song"] mode: Option<RepeatMode>,
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

    let current_track = call_lock
        .queue()
        .current()
        .ok_or("Nothing is currently playing to repeat.")?;

    match mode {
        Some(RepeatMode::Song) => {
            // Force loop ON
            current_track.enable_loop().ok();
            say!(ctx, "🔁 **Loop enabled** for the current track.");
            ctx_utils.end_loading_react().await?;
        }
        Some(RepeatMode::Disable) => {
            // Force loop OFF
            current_track.disable_loop().ok();
            say!(ctx, "➡️ **Loop disabled**");
            ctx_utils.end_loading_react().await?;
        }
        Some(RepeatMode::Toggle) => {
            // Get current state to decide whether to turn it on or off
            let track_info = current_track.get_info().await?;

            match track_info.loops {
                LoopState::Infinite => {
                    current_track.disable_loop().ok();
                    say!(ctx, "➡️ **Loop disabled*.");
                }
                _ => {
                    current_track.enable_loop().ok();
                    say!(ctx, "🔁 **Loop enabled**.");
                }
            }

            ctx_utils.end_loading_react().await?;
        }
        _ => {
            say!(
                ctx,
                "Invalid option! Use `song`, `disable`, `toggle`, or leave empty to toggle."
            );
            ctx_utils.end_loading_react().await?;
            ctx_utils
                .on_error_react(Some(&[ReactionType::Unicode("⁉️".into())]))
                .await?;
        }
    }

    Ok(())
}
