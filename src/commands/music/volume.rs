use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command)]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Enter percentage value from 0 to 200."]
    #[min = 0.0]
    #[max = 200.0]
    percentage: f32,
) -> Result<(), Error> {
    if percentage < 0.0 || percentage > 200.0 {
        ctx.say("Please enter a volume percentage between 0 and 200.")
            .await?;
        return Ok(());
    }

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("this commad ont works in servers.").await?;
        return Ok(());
    };

    let Some(user_ch) = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("you must be in a voice channel").await?;
        return Ok(());
    };

    let Some(client_ch) = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) else {
        ctx.say("I'm not in any channel").await?;
        return Ok(());
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(());
    };

    let Some(call) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(());
    };

    let track = {
        let handler = call.lock().await;
        handler.queue().current()
    };

    let Some(track) = track else {
        ctx.say("Nothing is in the queue to change volume.").await?;
        return Ok(());
    };

    let info = track.get_info().await?;

    if info.playing != songbird::tracks::PlayMode::Play {
        ctx.say("The music is must be already playing to change it's volume.")
            .await?;

        return Ok(());
    }

    track.set_volume(percentage / 100.0).ok();

    ctx.say(format!("changed the volume to {percentage}%"))
        .await?;

    Ok(())
}
