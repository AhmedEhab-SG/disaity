use poise::command;
use songbird::tracks::LoopState;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "repeat", aliases("rp"))]
pub async fn repeat(ctx: Context<'_>, #[rest] state: Option<String>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    // let Some(guild_id) = ctx.guild_id() else {
    //     ctx.say("this commad ont works in servers.").await?;
    //     return Ok(());
    // };

    let user_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("you must be in a voice channel").await?;
            return Ok(());
        }
    };

    let client_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("I'm not in any channel").await?;
            return Ok(());
        }
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let call = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.say("Not in a voice channel!").await?;
            return Ok(());
        }
    };

    let handler = call.lock().await;

    let queue = handler.queue();

    let Some(current_track) = queue.current() else {
        ctx.say("Nothing is currently playing to repeat.").await?;
        return Ok(());
    };

    // 2. Determine what action to take (On, Off, or Toggle)
    let action = state.unwrap_or_else(|| "toggle".to_string());

    match action.to_lowercase().as_str() {
        "on" | "enable" | "true" | "song" | "track" => {
            // Force loop ON
            let _ = current_track.enable_loop();
            ctx.say("🔁 **Loop enabled** for the current track.")
                .await?;
        }
        "off" | "disable" | "false" | "stop" => {
            // Force loop OFF
            let _ = current_track.disable_loop();
            ctx.say("➡️ **Loop disabled**.").await?;
        }
        "toggle" => {
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
        _ => {
            ctx.say("Invalid option! Use `on`, `off`, or leave empty to toggle.")
                .await?;
        }
    }
    Ok(())
}
