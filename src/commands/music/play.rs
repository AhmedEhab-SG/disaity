use std::sync::Arc;

use poise::{CreateReply, command};
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

#[command(slash_command, prefix_command, rename = "play", aliases("p"))]
pub async fn play(ctx: Context<'_>, #[rest] query: String) -> Result<(), Error> {
    ctx.channel_id()
        .broadcast_typing(&ctx.serenity_context().http)
        .await
        .ok();

    let do_search = !query.starts_with("http");

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
        return Ok(());
    };

    let voice_channel = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    });

    let Some(channel_id) = voice_channel else {
        ctx.say("You must be in a voice channel!").await?;
        return Ok(());
    };

    let Some(manager) = get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let call = get_or_join_voice(
        &manager,
        guild_id,
        channel_id,
        ctx.channel_id(),
        ctx.serenity_context().http.clone(),
        ctx.serenity_context().cache.clone(),
    )
    .await?;

    // handler.deafen(true).await?;

    let ctx_data = ctx.data();

    let mut src: Input = if do_search {
        YoutubeDl::new_search(ctx_data.http.clone(), query).into()
    } else {
        YoutubeDl::new(ctx_data.http.clone(), query).into()
    };

    let metadata = src.aux_metadata().await?;

    let song_info = SongMetadata {
        title: metadata.title.clone().unwrap_or("Unknown".to_string()),
        url: metadata
            .source_url
            .clone()
            .unwrap_or("https://youtube.com".to_string()),
        thumbnail: metadata.thumbnail.clone().unwrap_or_default(),
        duration: metadata.duration,
        request_by: ctx.author().name.clone(),
    };

    let track = Track::new_with_data(src.into(), Arc::new(song_info.clone()));

    let mut handler = call.lock().await;
    handler.enqueue(track).await;
    drop(handler);

    ctx.send(CreateReply::default().content(format!("Added to queue: {}", song_info.title)))
        .await?;

    Ok(())
}
