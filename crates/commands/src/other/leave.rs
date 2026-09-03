use poise::command;

use crate::checks::same_vc;
use disaity_core::{Context, Error};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "same_vc"
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx
        .guild_id()
        .ok_or("This command only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    manager.leave(guild_id).await?;

    ctx.say(
        ctx.data()
            .config
            .persona
            .interactions
            .events
            .guild
            .on_leave_vc
            .get_random_res()
            .unwrap_or("Bye"),
    )
    .await?;

    Ok(())
}
