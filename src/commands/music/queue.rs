use crate::core::{Context, Error};
use crate::handlers::SongMetadata;
use poise::{command, futures_util::StreamExt};
use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

#[command(slash_command, prefix_command)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let serenity_context = ctx.serenity_context();
    let cache = &serenity_context.cache;

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in servers.").await?;
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
        ctx.say("This command only works in servers.").await?;
        return Ok(());
    };

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("Not in a voice channel / no handler found").await?;
        return Ok(());
    };

    let queue_snapshot = {
        let handler = handler_lock.lock().await;
        handler.queue().current_queue()
    };

    if queue_snapshot.is_empty() {
        ctx.say("The queue is currently empty!").await?;
        return Ok(());
    }

    let mut tracks = Vec::new();

    for (i, handle) in queue_snapshot.iter().enumerate() {
        // Downcast to your stored metadata type (Arc<SongMetadata>)
        let track_info = handle.data::<SongMetadata>();

        let duration = track_info
            .duration
            .map(|d| format!("{}:{:02}", d.as_secs() / 60, d.as_secs() % 60))
            .unwrap_or_else(|| "??:??".to_string());

        let track_str = format!(
            "**{}.** [{}]({}) | `{}`\n*Requested by: {}*",
            i + 1,
            track_info.title,
            track_info.url,
            duration,
            track_info.request_by
        );

        tracks.push(track_str);
    }

    // 6) pagination settings
    let tracks_per_page = 5_usize;
    let total_pages = (tracks.len() as f32 / tracks_per_page as f32).ceil() as usize;
    let mut current_page = 0_usize;

    // 7) unique button ids (use the command ctx id to namespace them)
    let ctx_id = ctx.id();
    let ctx_id_str = ctx_id.to_string();
    let prev_buf = format!("{}_prev", ctx_id_str);
    let next_buf = format!("{}_next", ctx_id_str);

    // 8) helper closure to build the embed for a page
    // capture `queue_snapshot`, `tracks_per_page`, `total_pages` by move so closure can be used later
    let create_queue_embed = move |page: usize, list: &Vec<String>| {
        let start = page * tracks_per_page;
        let end = (start + tracks_per_page).min(list.len());

        let mut embed = CreateEmbed::new()
            .title("🎶 Current Queue")
            .description(list[start..end].join("\n\n"))
            .color(0x5865F2)
            .footer(CreateEmbedFooter::new(format!(
                "Page {} of {} | Total songs: {}",
                page + 1,
                total_pages,
                list.len()
            )));

        // Use the first track's thumbnail if available
        if let Some(first_handle) = queue_snapshot.first() {
            let info = first_handle.data::<SongMetadata>();
            embed = embed.thumbnail(&info.thumbnail);
        }

        embed
    };

    // 9) build components (buttons)
    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(&prev_buf)
            .label("◀️ Back")
            .style(serenity::all::ButtonStyle::Secondary),
        CreateButton::new(&next_buf)
            .label("Next ▶️")
            .style(serenity::all::ButtonStyle::Secondary),
    ])];

    // 10) send initial reply
    let builder = poise::CreateReply::default()
        .embed(create_queue_embed(0, &tracks))
        .components(components);

    ctx.send(builder).await?;

    // 11) create an interaction collector that filters only our buttons
    let mut interaction_stream = ComponentInteractionCollector::new(serenity_context)
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id_str))
        .timeout(std::time::Duration::from_secs(120)) // 2 minutes
        .stream();

    // 12) interaction loop — we are *not* holding the songbird lock here
    while let Some(mci) = interaction_stream.next().await {
        if mci.data.custom_id == prev_buf {
            current_page = if current_page == 0 {
                total_pages - 1
            } else {
                current_page - 1
            };
        } else if mci.data.custom_id == next_buf {
            current_page = (current_page + 1) % total_pages;
        }

        // Update the message with the new page embed
        mci.create_response(
            &ctx.serenity_context(),
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(create_queue_embed(current_page, &tracks)),
            ),
        )
        .await?;
    }

    Ok(())
}
