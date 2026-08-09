use poise::command;

use crate::checks::{diff_vc, not_mute, user_not_deafen};
use disaity_core::{Context, ContextExt, Error, VoiceUtils};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "diff_vc",
    check = "not_mute",
    check = "user_not_deafen"
)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let call = ctx.utils().get_or_join_voice().await?;

    let mut call_lock = call.lock().await;

    let queue = call_lock.queue();

    if !queue.is_empty() {
        queue.stop();
    }

    if ctx.author().id != ctx.data().config.info_registry.owner.id {
        call_lock.deafen(true).await?;
    }

    ctx.say("Yes?").await?;

    Ok(())
}
