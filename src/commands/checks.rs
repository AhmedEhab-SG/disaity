use crate::core::{context::Context, error::Error};

// pub async fn guild_cmd(ctx: Context<'_>) -> Result<bool, Error> {
//     if ctx.guild_id().is_none() {
//         ctx.say("This command only works in servers.").await?;
//         return Ok(false);
//     }
//     Ok(true)
// }

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
        (None, _) => {
            ctx.say("You must be in a voice channel").await?;
            Ok(false)
        }
        (_, None) => {
            ctx.say("I'm not in any channel").await?;
            Ok(false)
        }
        _ => {
            ctx.say("We must be in the same voice channel").await?;
            Ok(false)
        }
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
        (None, _) => {
            ctx.say("You need to be in a voice channel for me to join you!")
                .await?;
            Ok(false)
        }
        (Some(b), Some(u)) if b == u => {
            ctx.say("I'm already in your voice channel!").await?;
            Ok(false)
        }

        _ => Ok(true),
    }
}

pub async fn not_empty_queue(ctx: Context<'_>) -> Result<bool, Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
        return Ok(false);
    };

    let Some(manager) = songbird::get(ctx.serenity_context()).await else {
        ctx.say("failed to mount songbird").await?;
        return Ok(false);
    };

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel!").await?;
        return Ok(false);
    };

    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.is_empty() {
        ctx.say("The queue is already empty!").await?;
        return Ok(false);
    }

    Ok(true)
}

pub async fn not_deafen(ctx: Context<'_>) -> Result<bool, Error> {
    let is_deaf = ctx.guild().is_some_and(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .map(|state| state.deaf || state.self_deaf)
            .unwrap_or(false)
    });

    if is_deaf {
        ctx.say("You must be not deaffen to use that command")
            .await?;
        return Ok(false);
    }
    return Ok(true);
}
