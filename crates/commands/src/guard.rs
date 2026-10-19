use serenity::all::{Guild, Member, UserId};

use disaity_core::{Context, Error};

#[derive(Debug, Clone, Copy)]
pub struct Rank {
    pub id: UserId,
    pub top: u16,
    pub is_owner: bool,
}

impl Rank {
    pub fn of(guild: &Guild, member: &Member) -> Self {
        let is_owner = member.user.id == guild.owner_id;

        Self {
            id: member.user.id,
            top: if is_owner {
                u16::MAX
            } else {
                guild.member_highest_role(member).map_or(0, |r| r.position)
            },
            is_owner,
        }
    }
}

/// the author and the bot must both sit above the target, same rule discord uses
pub fn rank_rule(author: Rank, bot: Rank, target: Rank) -> Result<(), &'static str> {
    if target.id == author.id {
        return Err("You can't do that to yourself.");
    }

    if target.id == bot.id {
        return Err("I won't do that to myself.");
    }

    if target.is_owner {
        return Err("The server owner can't be targeted.");
    }

    if !author.is_owner && target.top >= author.top {
        return Err("Their highest role is equal to or above yours.");
    }

    if target.top >= bot.top {
        return Err("Their highest role is equal to or above mine.");
    }

    Ok(())
}

/// shared by kick, ban, timeout, remove_role... call it at the top of the body
pub async fn outranks(ctx: Context<'_>, target: &Member) -> Result<(), Error> {
    let author = ctx
        .author_member()
        .await
        .ok_or("This command only works in servers.")?
        .into_owned();

    let guild = ctx
        .guild()
        .ok_or("This command only works in servers.")?
        .clone();

    let bot = guild
        .member(ctx, ctx.framework().bot_id())
        .await?
        .into_owned();

    rank_rule(
        Rank::of(&guild, &author),
        Rank::of(&guild, &bot),
        Rank::of(&guild, target),
    )
    .map_err(Into::into)
}
