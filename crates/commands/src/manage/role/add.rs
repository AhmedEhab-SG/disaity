use poise::command;
use serenity::all::{Member, Role};

use disaity_core::{Context, Error, say};

use crate::Guard;

/// gives a member a role that sits below you and the bot
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_ROLES",
    required_bot_permissions = "MANAGE_ROLES | SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Who gets the role."] member: Member,
    #[description = "The role to give."] role: Role,
) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    Guard::role_below(ctx, &role).await?;

    if member.roles.contains(&role.id) {
        return Err(format!(
            "**{}** already has **{}**.",
            member.display_name(),
            role.name
        )
        .into());
    }

    member.add_role(ctx, role.id).await?;

    drop(typing);

    say!(
        ctx,
        format!(
            "➕ Gave **{}** the **{}** role",
            member.display_name(),
            role.name
        )
    );

    Ok(())
}
