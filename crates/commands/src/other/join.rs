use poise::command;

use disaity_core::{Context, ContextExt, Error, VoiceUtils};

use super::super::Check;

/// join voice channel
#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "Check::diff_vc",
    check = "Check::not_mute",
    check = "Check::user_not_deafen"
)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let call = ctx.utils().get_or_join_voice().await?;

    let mut call_lock = call.lock().await;

    let queue = call_lock.queue();

    if !queue.is_empty() {
        queue.stop();
    }

    call_lock
        .deafen(Some(ctx.author().id.into()) != ctx.data().config.info.owner.as_ref().map(|o| o.id))
        .await?;

    ctx.say(
        ctx.data()
            .config
            .persona
            .interactions
            .events
            .guild
            .on_join_vc
            .get_random_res()
            .unwrap_or("Yes"),
    )
    .await?;

    Ok(())
}
