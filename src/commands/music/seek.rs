use poise::command;
use songbird::tracks::PlayMode;

use crate::{
    core::{Context, Error},
    uitls::{format_duration_human, parse_timestamp},
};

#[command(slash_command, prefix_command)]
pub async fn seek(
    ctx: Context<'_>,
    #[description = "Enter time number or format you want to play in current song."] time: String,
) -> Result<(), Error> {
    let serenity_context = ctx.serenity_context();
    let cache = &serenity_context.cache;

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("this commad ont works in servers.").await?;
        return Ok(());
    };

    let Some(user_ch) = cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("you must be in a voice channel").await?;
        return Ok(());
    };

    let Some(client_ch) = cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("I'm not in any channel").await?;
        return Ok(());
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let Some(manager) = songbird::get(serenity_context).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(call) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let handler = call.lock().await;

    let Some(track) = handler.queue().current() else {
        ctx.say("Nothing is in the queue to seek.").await?;
        return Ok(());
    };

    let info = track.get_info().await?;

    if info.playing != PlayMode::Play {
        ctx.say("The music is must be already playing to seek a time frame in it")
            .await?;
        return Ok(());
    }

    let target = match parse_timestamp(&time) {
        Ok(dur) => dur,
        Err(msg) => {
            ctx.say(format!(
                "Invalid time: {}. Examples: `1:20`, `90`, `1m20s`, `1:02:30`",
                msg
            ))
            .await?;
            return Ok(());
        }
    };

    let result_time = track.seek(target).result_async().await?;

    ctx.say(format!("Seeked to {}", format_duration_human(result_time)))
        .await?;

    Ok(())
}
