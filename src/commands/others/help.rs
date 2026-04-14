use poise::{
    CreateReply, command,
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
};
use rand::seq::IndexedRandom;
use serenity::all::{
    CreateActionRow, CreateEmbedAuthor, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

use crate::{
    config::{characters::Character, commands::Command},
    core::{context::Context, error::Error},
};

#[command(
    slash_command,
    prefix_command,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let ctx_data = ctx.data();

    let character = ctx_data
        .config
        .characters_registry
        .get_character(&Character::Emilia);

    let interactions_registry = &ctx_data.config.interactions_registry;
    let built_quotes = &interactions_registry.messages.built;

    let commands_registry = &ctx_data.config.commands_registry;
    let help_command = commands_registry.get_command(&Command::Help);

    let (avatar_url, author_name) = {
        let user = ctx.serenity_context().cache.current_user();

        (
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
            format!("{} 💖", user.name),
        )
    };

    let select_menu = CreateSelectMenu::new(
        help_command
            .action
            .clone()
            .unwrap_or("help_menu".to_string()),
        CreateSelectMenuKind::String {
            options: commands_registry
                .categories
                .keys()
                .map(|cat| {
                    let cat_name = format!(
                        "{}{}",
                        &cat.to_string()[..1].to_string(),
                        &cat.to_string()[1..]
                    );
                    CreateSelectMenuOption::new(
                        format!("{} {}", commands_registry.get_cat_emoji(cat), cat_name),
                        cat_name.to_lowercase(),
                    )
                })
                .collect(),
        },
    )
    .placeholder(&help_command.description);

    let embed = CreateEmbed::new()
        .title("Hello,")
        .description(&character.summary)
        .thumbnail(&avatar_url)
        .color(interactions_registry.colors.help)
        .author(CreateEmbedAuthor::new(author_name).icon_url(avatar_url))
        .footer(
            CreateEmbedFooter::new(
                built_quotes
                    .choose(&mut rand::rng())
                    .unwrap_or(&"".to_string()),
            )
            .icon_url(&ctx_data.config.info_registry.owner.icon_url),
        );

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![CreateActionRow::SelectMenu(select_menu)]),
    )
    .await?;

    Ok(())
}
