use std::sync::Arc;

use poise::command;
use songbird::{input::YoutubeDl, tracks::Track};

use crate::{
    core::{Context, Error},
    handlers::{SongMetadata, playing::TrackStartNotifier},
};

fn format_duration(d: Option<std::time::Duration>) -> String {
    match d {
        Some(d) => {
            let seconds = d.as_secs();
            let minutes = seconds / 60;
            let rem_seconds = seconds % 60;
            format!("{:02}:{:02}", minutes, rem_seconds)
        }
        None => "Live/Unknown".to_string(),
    }
}

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

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = manager.join(guild_id, channel_id).await?;
    let mut handler = handler_lock.lock().await;

    let defer_msg = ctx.say("🔎 Searching...").await?;

    let client = reqwest::Client::new();

    let mut src: songbird::input::Input = if do_search {
        YoutubeDl::new_search(client, query).into()
    } else {
        YoutubeDl::new(client, query).into()
    };

    let metadata = match src.aux_metadata().await {
        Ok(m) => m,
        Err(_e) => {
            ctx.say("Could not fetch song metadata.").await?;
            return Ok(());
        }
    };

    let song_info = SongMetadata {
        title: metadata.title.clone().unwrap_or("Unknown".to_string()),
        url: metadata
            .source_url
            .clone()
            .unwrap_or("https://youtube.com".to_string()),
        thumbnail: metadata.thumbnail.clone().unwrap_or_default(),
        duration: metadata.duration,
    };

    let track = Track::new_with_data(src.into(), Arc::new(song_info.clone()));

    // let embed = CreateEmbed::new()
    //     .title(metadata.title.as_deref().unwrap_or("Unknown Title"))
    //     .url(
    //         metadata
    //             .source_url
    //             .as_deref()
    //             .unwrap_or("https://youtube.com"),
    //     )
    //     .thumbnail(metadata.thumbnail.as_deref().unwrap_or(""))
    //     .color(0xFF0000)
    //     .field("Duration", format_duration(metadata.duration), true)
    //     .field(
    //         "Channel",
    //         metadata.channel.as_deref().unwrap_or("Unknown"),
    //         true,
    //     )
    //     .footer(CreateEmbedFooter::new(format!(
    //         "Requested by {}",
    //         ctx.author().name
    //     )));

    // enqueue (songbird 0.5 uses `enqueue`)
    handler.enqueue(track).await;

    let _ = handler.add_global_event(
        songbird::Event::Track(songbird::TrackEvent::Play),
        TrackStartNotifier {
            channel_id: ctx.channel_id(),
            http: ctx.serenity_context().http.clone(),
        },
    );

    defer_msg
        .edit(
            ctx,
            poise::CreateReply::default().content(format!("Added to queue: {}", song_info.title)),
        )
        .await?;

    Ok(())
}
