use poise::command;
use serenity::all::{ChannelId, Member};

use disaity_core::{Context, Error, say};

/// moves a member into another voice channel
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MOVE_MEMBERS",
    required_bot_permissions = "MOVE_MEMBERS | CONNECT | SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn move_member(
    ctx: Context<'_>,
    #[description = "Who to move."] _member: Member,
    #[description = "The voice channel id to move them to."] _channel: ChannelId,
) -> Result<(), Error> {
    say!(ctx, "not implemented yet");

    Ok(())
}
