use poise::command;

use crate::{
    commands::{
        checks::{not_mute, user_not_deafen},
        macros::say,
    },
    core::{
        context::{Context, ContextExt},
        error::Error,
        utils::{ProviderUtils, ReactionUtils, VoiceUtils},
    },
};

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

    let (track, info) = ctx_utils.play(song).await?;

    if ctx.author().id != ctx.data().config.info_registry.owner.id {
        if call_lock.queue().len() <= 0 {
            call_lock.deafen(true).await?;
        }
    }

    call_lock.enqueue(track).await;

    ctx_utils.delete_self_reactions(&['🔍']).await?;
    ctx_utils.add_reactions(&['✅']).await?;

    say!(ctx, format!("**Fetched** {}", info.title), application_only);

    Ok(())
}
