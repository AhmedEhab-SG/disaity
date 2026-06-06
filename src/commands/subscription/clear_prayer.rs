use poise::command;

use crate::core::{context::Context, errors::Error};

#[command(
    slash_command,
    prefix_command,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn clear_prayer(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
