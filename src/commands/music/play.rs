use std::sync::Arc;

use poise::command;
use songbird::{
    input::{Input, YoutubeDl},
    tracks::Track,
};

use crate::{
    commands::{
        checks::{not_mute, user_not_deafen},
        macros::say,
    },
    core::{
        context::{Context, ContextExt},
        error::Error,
        utils::{ReactionUtils, VoiceUtils},
    },
    handlers::SongMetadata,
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
    let author = ctx.author();
    let do_search = !song.starts_with("http");

    ctx.defer().await?;

    let call = ctx.utils().get_or_join_voice().await?;

    ctx_utils.add_reactions(&['🔍']).await?;

    let mut src: Input = if do_search {
        YoutubeDl::new_search(ctx.data().http.clone(), song).into()
    } else {
        YoutubeDl::new(ctx.data().http.clone(), song).into()
    };

    let metadata = src.aux_metadata().await?;

    let song_info = SongMetadata {
        title: metadata
            .title
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        url: metadata
            .source_url
            .clone()
            .unwrap_or_else(|| "https://youtube.com".to_string()),
        thumbnail: metadata.thumbnail.clone().unwrap_or_default(),
        duration: metadata.duration,
        request_by: author.name.clone(),
        request_by_avatar: author
            .avatar_url()
            .unwrap_or_else(|| author.default_avatar_url()),
        author: metadata
            .channel
            .clone()
            .unwrap_or_else(|| "Unknown Author".to_string()),
        provider_logo_url: ctx
            .data()
            .config
            .interactions_registry
            .provider_logo_urls
            .youtube
            .clone(),
    };

    let track = Track::new_with_data(src.into(), Arc::new(song_info.clone()));

    let mut call_lock = call.lock().await;

    if author.id != ctx.data().config.info_registry.owner.id {
        if call_lock.queue().len() <= 0 {
            call_lock.deafen(true).await?;
        }
    }

    call_lock.enqueue(track).await;

    ctx_utils.delete_self_reactions(&['🔍']).await?;
    ctx_utils.add_reactions(&['✅']).await?;

    say!(
        ctx,
        format!("**Fetched** {}", song_info.title),
        application_only
    );

    Ok(())
}
