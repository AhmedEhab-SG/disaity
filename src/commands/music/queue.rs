use poise::{command, futures_util::StreamExt};

use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{
    commands::checks::{not_empty_queue, same_vc},
    core::{Context, ContextExt, Error, SongMetadata},
};

#[command(
    slash_command,
    prefix_command,
    guild_only,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | ADD_REACTIONS",
    check = "same_vc",
    check = "not_empty_queue"
)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let serenity_context = ctx.serenity_context();
    let ctx_utils = ctx.utils();

    let guild_id = ctx.guild_id().ok_or("this commad only works in servers.")?;

    let manager = songbird::get(serenity_context)
        .await
        .ok_or("failed to mount songbird")?;

    let call = manager
        .get(guild_id)
        .ok_or("Not in a voice channel / no handler found")?;

    ctx_utils.start_loading_react().await?;

    let queue_snapshot = {
        let _typing = ctx.defer_or_broadcast().await.ok().flatten();
        let call_lock = call.lock().await;
        call_lock.queue().current_queue()
    };

    let mut tracks = Vec::new();

    for (i, track_handle) in queue_snapshot.iter().enumerate() {
        // Downcast to your stored metadata type (Arc<SongMetadata>)
        let track_info = track_handle.data::<SongMetadata>();

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

    ctx_utils.end_loading_react().await?;

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
