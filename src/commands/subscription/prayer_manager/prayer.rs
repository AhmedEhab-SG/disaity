use poise::command;
use serenity::{
    all::ChannelType,
    model::id::{ChannelId, RoleId},
};

use crate::{
    commands::subscription::prayer_manager::PrayerSubscriptionInfo,
    core::{
        context::{Context, ContextExt},
        errors::Error,
        macros::say,
    },
};

#[command(
    slash_command,
    prefix_command,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn prayer(
    ctx: Context<'_>,
    #[description = "The city name (e.g., Cairo, Riyadh, New York)"] city: String,
    #[description = "The country name (e.g., Egypt, Saudi Arabia, USA)"] country: String,
    #[description = "Enter the channel id you want to send the notification on."]
    channel_id: ChannelId,
    #[description = "Add role id that u want to ping! (Optional)"] role_id: Option<RoleId>,
) -> Result<(), Error> {
    let ctx_utils = ctx.utils();

    let guild = ctx
        .guild()
        .ok_or("must be in guild to use that command")?
        .clone();

    let channel = guild
        .channels
        .get(&channel_id)
        .cloned()
        .ok_or("failed to find the channel.")?;

    let role_id = if let Some(r_id) = role_id {
        Some(
            guild
                .roles
                .get(&r_id)
                .cloned()
                .ok_or("failed to find the role.")?
                .id,
        )
    } else {
        None
    };

    if channel.kind != ChannelType::Text {
        return Err("you can subscribe only on a text channel".into());
    }

    ctx_utils.start_loading_react().await?;

    let mut prayer_sub = ctx.data().subscription.prayer_subscription.write().await;

    prayer_sub.add_subscription(
        guild.id,
        PrayerSubscriptionInfo {
            channel_id,
            role_id,
            city: city.trim().to_string(),
            country: country.trim().to_string(),
        },
    );

    ctx_utils.end_loading_react().await?;

    channel
        .say(
            ctx.http(),
            format!(
                "🕌 Prayer notifications successfully configured for **{}, {}** in this channel!",
                city, country
            ),
        )
        .await?;

    say!(
        ctx,
        format!("subscribed the prayer time on **{}**", channel.name)
    );

    Ok(())
}
