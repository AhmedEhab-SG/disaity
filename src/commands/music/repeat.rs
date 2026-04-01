use poise::{ChoiceParameter, command};
use songbird::tracks::LoopState;

use crate::core::{Context, Error};

#[derive(Debug, ChoiceParameter)]
enum RepeatMode {
    #[name = "song"]
    Song,
    #[name = "toggle"]
    Toggle,
    #[name = "disable"]
    Disable,
}

#[command(slash_command, prefix_command)]
pub async fn repeat(
    ctx: Context<'_>,
    #[description = "Repeat's a song"] mode: Option<RepeatMode>,
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

    let queue = handler.queue();

    let Some(current_track) = queue.current() else {
        ctx.say("Nothing is currently playing to repeat.").await?;
        return Ok(());
    };

    match mode {
        Some(RepeatMode::Song) => {
            // Force loop ON
            let _ = current_track.enable_loop();
            ctx.say("🔁 **Loop enabled** for the current track.")
                .await?;
        }
        Some(RepeatMode::Disable) => {
            // Force loop OFF
            let _ = current_track.disable_loop();
            ctx.say("➡️ **Loop disabled**.").await?;
        }
        Some(RepeatMode::Toggle) => {
            // Get current state to decide whether to turn it on or off
            let track_info = current_track.get_info().await?;

            match track_info.loops {
                LoopState::Infinite => {
                    let _ = current_track.disable_loop();
                    ctx.say("➡️ **Loop disabled**.").await?;
                }
                _ => {
                    let _ = current_track.enable_loop();
                    ctx.say("🔁 **Loop enabled**.").await?;
                }
            }
        }
        None => {
            ctx.say("Invalid option! Use `song`, `disable`, `toggle`, or leave empty to toggle.")
                .await?;
        }
    }
    Ok(())
}
