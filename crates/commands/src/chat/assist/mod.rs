use poise::command;

use disaity_core::{Context, Error, say};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 10,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
pub async fn assist(
    ctx: Context<'_>,
    #[description = "Tell me what you want, Will help you."]
    #[rest]
    _request: String,
) -> Result<(), Error> {
    say!(ctx, "not implemented yet");

    Ok(())
}
