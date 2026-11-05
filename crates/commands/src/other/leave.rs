use poise::command;

use disaity_core::{Context, Error, say};

use super::super::Check;

/// leave voice channel and distroy call hooks
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "Check::same_vc"
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx
        .guild_id()
        .ok_or("This command only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    manager.remove(guild_id).await?;

    drop(typing);

    say!(
        ctx,
        ctx.data()
            .config
            .persona
            .interactions
            .events
            .guild
            .on_leave_vc
            .get_random_res()
            .unwrap_or("Bye")
    );

    Ok(())
}
