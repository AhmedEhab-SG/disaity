use poise::command;

use disaity_core::{Context, ContextExt, Error, say};

use super::store::FreeGamesStore;

#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let ctx_utils = ctx.utils();

    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx
        .guild_id()
        .ok_or("must be in guild to use that command")?;

    ctx_utils.start_loading_react().await?;

    let db = ctx
        .data()
        .db
        .as_ref()
        .ok_or("Subscriptions are not configured on this bot.")?;

    let removed = FreeGamesStore::new(db.pool.clone())
        .delete(guild_id)
        .await?;

    ctx_utils.end_loading_react().await?;

    drop(typing);

    if removed {
        say!(
            ctx,
            "❌ Successfully disabled and cleared free games notifications for this server."
        );
    } else {
        say!(
            ctx,
            "⚠️ There is no active free games subscription configured for this server."
        );
    }

    Ok(())
}
