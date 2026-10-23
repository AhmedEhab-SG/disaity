use poise::command;
use serenity::all::{ChannelId, ChannelType, Member};

use disaity_core::{Context, Error, say};

use super::super::guard::Guard;

/// moves a member into another voice channel
#[command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MOVE_MEMBERS",
    required_bot_permissions = "MOVE_MEMBERS | CONNECT | SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn move_member(
    ctx: Context<'_>,
    #[description = "Who to move."] member: Member,
    #[description = "The voice channel id to move them to."] channel: ChannelId,
) -> Result<(), Error> {
    // moving yourself is fine, moving someone else needs a higher role
    if member.user.id != ctx.author().id {
        Guard::outranks(ctx, &member).await?;
    }

    let (name, in_voice) = {
        let guild = ctx.guild().ok_or("This command only works in servers.")?;

        let target = guild
            .channels
            .get(&channel)
            .ok_or("I can't find that channel.")?;

        if !matches!(target.kind, ChannelType::Voice | ChannelType::Stage) {
            return Err("That's not a voice channel.".into());
        }

        let in_voice = guild
            .voice_states
            .get(&member.user.id)
            .is_some_and(|vs| vs.channel_id.is_some());

        (target.name.clone(), in_voice)
    };

    // discord can only move people who are already connected somewhere
    if !in_voice {
        return Err(format!("**{}** isn't in a voice channel.", member.display_name()).into());
    }

    member
        .guild_id
        .move_member(ctx, member.user.id, channel)
        .await?;

    say!(
        ctx,
        format!("🔀 Moved **{}** to **{name}**", member.display_name())
    );

    Ok(())
}
