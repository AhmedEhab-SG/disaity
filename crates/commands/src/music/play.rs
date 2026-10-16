use poise::command;
use tokio::time::{Duration, timeout};

use disaity_config::CommandId;
use disaity_core::{Context, ContextExt, Error, ReactionUtils, VoiceUtils, say};

use super::super::checks::{not_mute, user_not_deafen};

/// takes a list or track to join the channel and start playing
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT | SPEAK | EMBED_LINKS | ADD_REACTIONS",
    check = "not_mute",
    check = "user_not_deafen"
)]
pub async fn play(
    ctx: Context<'_>,

    #[description = "Enter song name"]
    #[rest]
    song: String,
) -> Result<(), Error> {
    let ctx_utils = ctx.utils();

    ctx.defer().await?;

    let call = ctx.utils().get_or_join_voice().await?;

    ctx_utils.add_reactions(&['🔍']).await?;

    let mut call_lock = call.lock().await;

    if ctx.author().id != ctx.data().config.info.owner.id && call_lock.queue().is_empty() {
        call_lock.deafen(true).await?;
    }
    // free the call to prevent stucking until the fetching happens
    drop(call_lock);

    let timeout_duration = ctx
        .data()
        .config
        .commands_registry
        .get_command(&CommandId::Play)
        .timeout
        .unwrap_or(30);

    // add timeout if the trafic stuck
    let queued = timeout(Duration::from_secs(timeout_duration), ctx_utils.play(song, call))
        .await
        .map_err(|_|format!( "The playlist or song took too long to load ({timeout_duration}s limit). Try a shorter query!"))??;

    ctx_utils.delete_self_reactions(&['🔍']).await?;
    ctx_utils.add_reactions(&['✅']).await?;

    let response = if queued.total > 1 {
        format!(
            "**Fetched** {} and {} more from its playlist",
            queued.first_title,
            queued.total - 1
        )
    } else {
        format!("**Fetched** {}", queued.first_title)
    };

    say!(ctx, response, application_only);

    Ok(())
}
