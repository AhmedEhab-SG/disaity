use std::sync::Arc;

use poise::{Context as MessageContext, command};
use songbird::{
    get,
    input::{Input, YoutubeDl},
    tracks::Track,
};

use crate::{
    commands::get_or_join_voice,
    core::{Context, Error},
    handlers::SongMetadata,
};

#[command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Enter song name"]
    #[rest]
    song: String,
) -> Result<(), Error> {
    let serenity_context = ctx.serenity_context();
    let text_channel_id = ctx.channel_id();
    let author = ctx.author();
    let http = &serenity_context.http;
    let cache = &serenity_context.cache;

    let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let do_search = !song.starts_with("http");

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
        return Ok(());
    };

    let Some(voice_channel_id) = cache
        .guild(guild_id)
        .and_then(|g| g.voice_states.get(&author.id).and_then(|vs| vs.channel_id))
    else {
        ctx.say("You must be in a voice channel!").await?;
        return Ok(());
    };

    let Some(manager) = get(serenity_context).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let call = get_or_join_voice(
        manager,
        guild_id,
        voice_channel_id,
        text_channel_id,
        http,
        cache,
    )
    .await?;

    // handler.deafen(true).await?;

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

    let mut handler = call.lock().await;

    handler.enqueue(track).await;

    match ctx {
        MessageContext::Application(_) => {
            ctx.say(format!("Fetched {}", song_info.title)).await.ok()
        }
        MessageContext::Prefix(_) => None,
    };

    Ok(())
}
