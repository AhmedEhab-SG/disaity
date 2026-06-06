use poise::command;

use crate::core::{
    context::{Context, ContextExt},
    errors::Error,
    macros::say,
};

#[command(
    slash_command,
    prefix_command,
    broadcast_typing,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn clear_prayer(ctx: Context<'_>) -> Result<(), Error> {
    let ctx_utils = ctx.utils();

    let guild_id = ctx
        .guild_id()
        .ok_or("must be in guild to use that command")?;

    ctx_utils.start_loading_react().await?;

    let mut prayer_sub = ctx.data().subscription.prayer_subscription.write().await;

    let removed = prayer_sub.remove_subscription(guild_id);

    ctx_utils.end_loading_react().await?;

    if removed.is_some() {
        say!(
            ctx,
            "❌ Successfully disabled and cleared prayer time notifications for this server."
        );
    } else {
        say!(
            ctx,
            "⚠️ There is no active prayer time subscription configured for this server."
        );
    }

    Ok(())
}
