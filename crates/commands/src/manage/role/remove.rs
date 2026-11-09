use poise::command;
use serenity::all::{Member, Role};

use disaity_core::{Context, Error, say};

use crate::Guard;

/// takes a role that sits below you and the bot from a member
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_ROLES",
    required_bot_permissions = "MANAGE_ROLES | SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Who loses the role."] member: Member,
    #[description = "The role to take."] role: Role,
) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    Guard::role_below(ctx, &role).await?;

    if !member.roles.contains(&role.id) {
        return Err(format!(
            "**{}** doesn't have **{}**.",
            member.display_name(),
            role.name
        )
        .into());
    }

    member.remove_role(ctx, role.id).await?;

    drop(typing);

    say!(
        ctx,
        format!(
            "➖ Took **{}** from **{}**",
            role.name,
            member.display_name()
        )
    );

    Ok(())
}
