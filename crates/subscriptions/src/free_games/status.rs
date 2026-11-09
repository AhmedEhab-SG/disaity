use poise::command;

use disaity_core::{Context, Error, say};

use super::store::FreeGamesStore;

/// shows where free games are posted in this server
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx
        .guild_id()
        .ok_or("must be in guild to use that command")?;

    let db = ctx
        .data()
        .db
        .as_ref()
        .ok_or("Subscriptions are not configured on this bot.")?;

    let Some(info) = FreeGamesStore::new(db.pool.clone()).get(guild_id).await? else {
        return Err(
            "⚠️ Free games aren't set up in this server. Use `free_games set` to start.".into(),
        );
    };

    // the role's name, not a mention, so checking doesn't ping anyone
    let role = info
        .role_id
        .and_then(|id| Some(ctx.guild()?.roles.get(&id)?.name.clone()))
        .map_or("no role".to_string(), |name| format!("the **{name}** role"));

    drop(typing);

    say!(
        ctx,
        format!(
            "🎮 Free games are posted in <#{}> and ping {role}.",
            info.channel_id
        )
    );

    Ok(())
}
