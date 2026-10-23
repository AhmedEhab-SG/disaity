use poise::command;
use serenity::all::Member;

use disaity_core::{Context, Error, say};

use super::super::{guard::Guard, utils::audit_reason};

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
    #[description = "Who to kick."] member: Member,
    #[description = "Why, shown in the audit log."]
    #[rest]
    reason: String,
) -> Result<(), Error> {
    Guard::outranks(ctx, &member).await?;

    member
        .kick_with_reason(ctx, &audit_reason(ctx, &reason).await)
        .await?;

    say!(
        ctx,
        format!("👢 Kicked **{}**: {reason}", member.display_name())
    );

    Ok(())
}
