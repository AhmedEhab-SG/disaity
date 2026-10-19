use poise::command;
use serenity::all::Member;

use disaity_core::{Context, Error, say};

/// kicks a member who sits below you and the bot
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS | SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Who to kick."] _member: Member,
    #[description = "Why, shown in the audit log."]
    #[rest]
    _reason: String,
) -> Result<(), Error> {
    say!(ctx, "not implemented yet");

    Ok(())
}
