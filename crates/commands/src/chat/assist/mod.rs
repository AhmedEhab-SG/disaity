mod call;
mod confirm;
mod consts;
mod lookup;
mod tool;

use poise::command;

use disaity_core::{Context, Error, say};

use call::Call;
use tool::Tool;

/// understands a casual request and runs your own commands for it
#[command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 10,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
pub async fn assist(
    ctx: Context<'_>,
    #[description = "Tell me what you want, in any words."]
    #[rest]
    request: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let ai = data
        .ai
        .as_ref()
        .ok_or("AI is not configured on this bot.")?;

    let persona = &data.config.persona;

    let prefix = &data.config.info.prefix;

    let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    // every command uses ai flag, turned into something gemini can call
    let commands = ctx
        .framework()
        .options()
        .commands
        .iter()
        .filter_map(Tool::from_command)
        .collect::<Vec<Tool>>();

    // the lookups and your commands, the whole menu gemini picks from
    let declarations = Call::declarations(&commands);

    // the message every command runs from, made once per request
    let msg = Tool::carrier(ctx).await?;

    let author_perms = {
        let member = ctx
            .author_member()
            .await
            .ok_or("This command only works in servers.")?;

        ctx.guild()
            .ok_or("This command only works in servers.")?
            .member_permissions(&member)
    };

    say!(ctx, "not implemented yet");

    Ok(())
}
