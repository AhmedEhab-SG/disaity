use std::sync::Arc;

use crate::uitls::get_or_join_voice;

use poise::command;
use songbird::{input::YoutubeDl, tracks::Track};

use crate::{
    core::{Context, Error},
    handlers::SongMetadata,
};

#[command(slash_command, prefix_command, rename = "play", aliases("p"))]
pub async fn play(ctx: Context<'_>, #[rest] query: String) -> Result<(), Error> {
    let do_search = !query.starts_with("http");

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
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

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let call = get_or_join_voice(
        &manager,
        guild_id,
        channel_id,
        ctx.channel_id(),
        ctx.serenity_context().http.clone(),
    )
    .await?;

    let mut handler = call.lock().await;

    // handler.deafen(true).await?;

    let defer_msg = ctx.say("🔎 Searching...").await?;

    let client = reqwest::Client::new();

    let mut src: songbird::input::Input = if do_search {
        YoutubeDl::new_search(client, query).into()
    } else {
        YoutubeDl::new(client, query).into()
    };

    let Ok(metadata) = src.aux_metadata().await else {
        ctx.say("Could not fetch song metadata.").await?;
        return Ok(());
    };

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

    handler.enqueue(track).await;

    defer_msg
        .edit(
            ctx,
            poise::CreateReply::default().content(format!("Added to queue: {}", song_info.title)),
        )
        .await?;

    Ok(())
}
