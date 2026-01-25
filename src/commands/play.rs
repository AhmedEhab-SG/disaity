use poise::command;
use songbird::input::YoutubeDl;

use crate::core::{Context, Error};

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

    // create a reqwest client (share it if you're making many requests)
    let client = reqwest::Client::new();

    // Build YoutubeDl source (note: takes Client + url)
    let source = if do_search {
        YoutubeDl::new_search(client, query).into()
    } else {
        YoutubeDl::new(client, query).into()
    };

    // enqueue (songbird 0.5 uses `enqueue`)
    handler.enqueue(source).await;

    ctx.say("▶️ Added to queue!").await?;
    Ok(())
}
