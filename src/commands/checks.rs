use crate::core::{Context, Error};

pub async fn dm_with_auth(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.guild_id().is_some() && ctx.author().id != ctx.data().config.info_registry.owner.id {
        return Err("You are not authorized to use this command in DMs.".into());
    }
    Ok(true)
}

pub async fn same_vc(ctx: Context<'_>) -> Result<bool, Error> {
    let (user_vc, client_vc) = {
        let guild = ctx.guild().ok_or("This command only works in servers.")?;

        let c = guild
            .voice_states
            .get(&ctx.framework().bot_id)
            .and_then(|vs| vs.channel_id);

        let u = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id);

        (u, c)
    };

    match (user_vc, client_vc) {
        (Some(c), Some(u)) if c == u => Ok(true),
        (None, _) => return Err("You must be in a voice channel".into()),
        (_, None) => return Err("I'm not in any channel".into()),
        _ => return Err("We must be in the same voice channel".into()),
    }
}

pub async fn diff_vc(ctx: Context<'_>) -> Result<bool, Error> {
    let (user_vc, client_vc) = {
        let guild = ctx.guild().ok_or("This command only works in servers.")?;

        let c = guild
            .voice_states
            .get(&ctx.framework().bot_id)
            .and_then(|vs| vs.channel_id);

        let u = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id);

        (u, c)
    };

    match (user_vc, client_vc) {
        (None, _) => return Err("You need to be in a voice channel for me to join you!".into()),
        (Some(b), Some(u)) if b == u => return Err("I'm already in your voice channel!".into()),
        _ => Ok(true),
    }
}

pub async fn not_empty_queue(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager.get(guild_id).ok_or("Not in a voice channel!")?;

    let call_lock = call.lock().await;

    let queue = call_lock.queue();

    if queue.is_empty() {
        return Err("The queue is already empty!".into());
    }

    Ok(true)
}

pub async fn user_not_deafen(ctx: Context<'_>) -> Result<bool, Error> {
    let is_deaf = ctx.guild().is_some_and(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .map(|state| state.deaf || state.self_deaf)
            .unwrap_or(false)
    });

    if is_deaf {
        return Err("You must be not deaffen to use that command".into());
    }
    return Ok(true);
}

pub async fn not_mute(ctx: Context<'_>) -> Result<bool, Error> {
    let client_id = ctx.cache().current_user().id;

    let is_muted = ctx.guild().is_some_and(|g| {
        g.voice_states
            .get(&client_id)
            .map(|state| state.mute || state.self_mute)
            .unwrap_or(false)
    });

    if is_muted {
        return Err("🔇 **I'm currently muted.** Please unmute the me to use this command.".into());
    }

    Ok(true)
}
