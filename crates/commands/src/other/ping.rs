use poise::command;

use disaity_core::{Context, Error, say};

/// ping the bot to pong you
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    drop(typing);

    say!(ctx, format!("Pong!"));

    Ok(())
}
