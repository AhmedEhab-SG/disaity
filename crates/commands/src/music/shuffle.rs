use poise::command;

use rand::{rng, seq::SliceRandom};

use disaity_core::{Context, ContextExt, Error, say};

use crate::checks::{not_empty_queue, not_mute, same_vc, user_not_deafen};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "same_vc",
    check = "not_empty_queue",
    check = "not_mute",
    check = "user_not_deafen"
)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    let ctx_utils = ctx.utils();
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager.get(guild_id).ok_or("Not in a voice channel!")?;

    ctx_utils.start_loading_react().await?;

    let call_lock = call.lock().await;

    let queue = call_lock.queue();

    if queue.len() <= 1 {
        return Err("You can't shuffle a queue with one song or none!.".into());
    };

    queue.modify_queue(|q| {
        let mut rest: Vec<_> = q.drain(1..).collect();

        rest.shuffle(&mut rng());

        q.extend(rest.into_iter());
    });

    ctx_utils.end_loading_react().await?;

    say!(ctx, "Shuffled the queue.");

    Ok(())
}
