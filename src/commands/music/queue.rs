use crate::core::{Context, Error};
use crate::handlers::SongMetadata;
use poise::futures_util::StreamExt;
use poise::{command, serenity_prelude as serenity};

use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

#[command(slash_command, prefix_command, rename = "queue", aliases("q"))]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    // 1. Get Songbird Manager
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird not initialized")?;

    let handler_lock = manager.get(guild_id).ok_or("I'm not in a voice channel")?;
    let handler = handler_lock.lock().await;

    // 2. Get the current queue
    let queue = handler.queue().current_queue();
    if queue.is_empty() {
        ctx.say("The queue is currently empty!").await?;
        return Ok(());
    }

    // 3. Extract Metadata from your custom SongMetadata struct
    let mut tracks = Vec::new();
    for (i, handle) in queue.iter().enumerate() {
        // We must downcast to Arc<SongMetadata> because that's how you stored it in play.rs
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

    // 4. Pagination Settings
    let tracks_per_page = 5;
    let total_pages = (tracks.len() as f32 / tracks_per_page as f32).ceil() as usize;
    let mut current_page = 0;

    // Unique IDs for buttons based on this specific command message
    let ctx_id = ctx.id();
    let prev_buf = format!("{}_prev", ctx_id);
    let next_buf = format!("{}_next", ctx_id);

    // Helper to generate the embed for a specific page
    let create_queue_embed = |page: usize, list: &Vec<String>| {
        let start = page * tracks_per_page;
        let end = (start + tracks_per_page).min(list.len());

        // Grab the thumbnail of the first song in the queue for the embed
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

        // If the first track exists, show its thumbnail
        if let Some(first_handle) = queue.first() {
            let info = first_handle.data::<SongMetadata>();
            embed = embed.thumbnail(&info.thumbnail)
        };
        embed
    };

    // 5. Build Initial Reply
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

    ctx.send(builder).await?;

    // 6. Interaction Loop (Button Clicks)
    let mut interaction_stream = ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(120)) // 2 minute timeout
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

        // Update the message with the new page
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
