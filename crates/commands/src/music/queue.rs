use poise::{command, futures_util::StreamExt};

use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

use disaity_core::{Context, ContextExt, Error, TrackMetadata, send};

use super::super::checks::{not_empty_queue, same_vc};

/// generate paginated queue
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

    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let call_lock = call.lock().await;

    let queue_snapshot = call_lock.queue().current_queue();

    // release the call lock beacuse the response will still hold the call
    drop(call_lock);

    let mut tracks = Vec::new();

    let now_playing = queue_snapshot
        .first()
        .map(|handle| handle.data::<TrackMetadata>().title.clone())
        .unwrap_or_default();

    for (i, track_handle) in queue_snapshot.iter().enumerate() {
        let track_info = track_handle.data::<TrackMetadata>();

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

    let tracks_per_page = 5_usize;
    let total_pages = (tracks.len() as f32 / tracks_per_page as f32).ceil() as usize;
    let mut current_page = 0_usize;

    let ctx_id = ctx.id();
    let ctx_id_str = ctx_id.to_string();
    let prev_buf = format!("{}_prev", ctx_id_str);
    let next_buf = format!("{}_next", ctx_id_str);

    // helper pagination
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

        if let Some(first_handle) = queue_snapshot.first() {
            let info = first_handle.data::<TrackMetadata>();
            embed = embed.thumbnail(&info.thumbnail);
        }

        embed
    };

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(&prev_buf)
            .label("◀️ Back")
            .style(serenity::all::ButtonStyle::Secondary),
        CreateButton::new(&next_buf)
            .label("Next ▶️")
            .style(serenity::all::ButtonStyle::Secondary),
    ])];

    let builder = poise::CreateReply::default()
        .embed(create_queue_embed(0, &tracks))
        .components(components);

    ctx_utils.end_loading_react().await?;

    // relaase typing interaction before the stream
    drop(typing);

    if send!(
        ctx,
        builder,
        format!("{} tracks queued, now playing {now_playing}", tracks.len())
    )
    .is_none()
    {
        // exit the command if its none to dont wait for the whole time
        return Ok(());
    }

    // create an interaction collector that filters only our buttons
    let mut interaction_stream = ComponentInteractionCollector::new(serenity_context)
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id_str))
        .timeout(std::time::Duration::from_secs(120)) // 2 minutes
        .stream();

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
