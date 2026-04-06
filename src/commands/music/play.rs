use std::sync::Arc;

use poise::command;
use songbird::{
    get,
    input::{Input, YoutubeDl},
    tracks::Track,
};

use crate::{
    commands::checks::not_deafen,
    core::{
        context::{Context, ContextExt},
        error::Error,
        utils::UtilsExt,
    },
    handlers::SongMetadata,
};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT | SPEAK | EMBED_LINKS | ADD_REACTIONS",
    check = "not_deafen"
)]
pub async fn play(
    ctx: Context<'_>,

    #[description = "Enter song name"]
    #[rest]
    song: String,
) -> Result<(), Error> {
    let author = ctx.author();
    let do_search = !song.starts_with("http");

    let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let Some(manager) = get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let call = ctx.utils().get_or_join_voice(manager).await?;

    if let Context::Prefix(p_ctx) = ctx {
        p_ctx.msg.react(ctx, '🔍').await?;
    }

    let mut src: Input = if do_search {
        YoutubeDl::new_search(ctx.data().http.clone(), song).into()
    } else {
        YoutubeDl::new(ctx.data().http.clone(), song).into()
    };

    let metadata = src.aux_metadata().await?;

    let song_info = SongMetadata {
        title: metadata.title.unwrap_or("Unknown".to_string()),
        url: metadata
            .source_url
            .unwrap_or("https://youtube.com".to_string()),
        thumbnail: metadata.thumbnail.unwrap_or_default(),
        duration: metadata.duration,
        request_by: author.name.clone(),
    };

    let track = Track::new_with_data(src.into(), Arc::new(song_info.clone()));

    let mut call_lock = call.lock().await;

    if author.id != ctx.data().config.info_registry.owner.id {
        if call_lock.queue().len() <= 0 {
            call_lock.deafen(true).await?;
        }
    }

    call_lock.enqueue(track).await;

    match ctx {
        Context::Application(_) => {
            let reply_handle = ctx.say(format!("Fetched {}", song_info.title)).await?;
            let msg = reply_handle.message().await?;
            msg.react(ctx, '✅').await?;
        }

        Context::Prefix(p_ctx) => {
            p_ctx.msg.delete_reactions(ctx).await?;
            p_ctx.msg.react(ctx, '✅').await?;
        }
    };

    Ok(())
}
