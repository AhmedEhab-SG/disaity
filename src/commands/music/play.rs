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

    let tracks_data = ctx_utils.play(song).await?;

    if ctx.author().id != ctx.data().config.info_registry.owner.id {
        if call_lock.queue().len() <= 0 {
            call_lock.deafen(true).await?;
        }
    }

    let is_playlist = tracks_data.len() > 1;

    let first_title = tracks_data
        .first()
        .map(|(_, info)| info.title.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    for (track, _) in tracks_data {
        call_lock.enqueue(track).await;
    }

    ctx_utils.delete_self_reactions(&['🔍']).await?;
    ctx_utils.add_reactions(&['✅']).await?;

    let response = if is_playlist {
        format!("**Fetched** {} and its playlist", first_title)
    } else {
        format!("**Fetched** {}", first_title)
    };

    say!(ctx, response, application_only);

    Ok(())
}
