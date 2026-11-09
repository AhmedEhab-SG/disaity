use serenity::all::{Guild, Member, Role, UserId};

use disaity_core::{Context, Error};

pub struct Guard;

impl Guard {
    /// the author and the bot must both sit above the target, same rule discord uses
    fn rank_rule(author: Rank, bot: Rank, target: Rank) -> Result<(), &'static str> {
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

    /// the role must sit below the author's and the bot's highest role, same rule discord uses
    fn role_rule(author: Rank, bot: Rank, position: u16) -> Result<(), &'static str> {
        if !author.is_owner && position >= author.top {
            return Err("That role is equal to or above your highest role.");
        }

        if position >= bot.top {
            return Err("That role is equal to or above my highest role.");
        }

        Ok(())
    }

    /// the guild, with the author's and the bot's rank in it
    async fn ranks(ctx: Context<'_>) -> Result<(Guild, Rank, Rank), Error> {
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

        let author = Rank::of(&guild, &author);
        let bot = Rank::of(&guild, &bot);

        Ok((guild, author, bot))
    }

    /// shared by kick, ban, timeout, remove_role... call it at the top of the body
    pub async fn outranks(ctx: Context<'_>, target: &Member) -> Result<(), Error> {
        let (guild, author, bot) = Self::ranks(ctx).await?;

        Self::rank_rule(author, bot, Rank::of(&guild, target)).map_err(Into::into)
    }

    /// for role add and remove: only the role's place counts, not the member's.
    /// @everyone and integration roles can't be given or taken at all
    pub async fn role_below(ctx: Context<'_>, role: &Role) -> Result<(), Error> {
        if role.managed {
            return Err("That role belongs to an integration or a bot.".into());
        }

        if role.id.get() == role.guild_id.get() {
            return Err("Everyone has the @everyone role.".into());
        }

        let (_, author, bot) = Self::ranks(ctx).await?;

        Self::role_rule(author, bot, role.position).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy)]
struct Rank {
    pub id: UserId,
    pub top: u16,
    pub is_owner: bool,
}

impl Rank {
    fn of(guild: &Guild, member: &Member) -> Self {
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
