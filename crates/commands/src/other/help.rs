use std::{str::FromStr, time::Duration};

use poise::{
    CreateReply, command,
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
};
use serenity::{
    all::{
        ComponentInteractionCollector, ComponentInteractionDataKind, CreateActionRow,
        CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    },
    futures::StreamExt,
    model::id::UserId,
};

use disaity_config::{Category, CommandId};
use disaity_core::{CommandUtils, Context, ContextExt, Error};

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();

    chars
        .next()
        .map(|first_char| first_char.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// present all the commands
#[command(
    slash_command,
    prefix_command,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let serenity_context = ctx.serenity_context();
    let ctx_data = ctx.data();

    let persona = &ctx_data.config.persona;
    let info = &ctx_data.config.info;

    let commands_registry = &ctx_data.config.commands_registry;
    let by_category = ctx.utils().commands_by_category();

    let (avatar_url, author_name) = {
        let user = serenity_context.cache.current_user();

        (
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
            format!("{} 💖", user.name),
        )
    };

    let cat_emoji = |cat: &str| {
        Category::from_str(cat)
            .map(|c| commands_registry.get_cat_emoji(&c))
            .unwrap_or("📁")
    };

    let select_menu = CreateSelectMenu::new(
        commands_registry
            .get_command(&CommandId::Help)
            .action
            .clone()
            .unwrap_or("help_menu".to_string()),
        CreateSelectMenuKind::String {
            options: by_category
                .keys()
                .map(|cat| {
                    CreateSelectMenuOption::new(
                        format!("{} {}", cat_emoji(cat), capitalize(cat)),
                        *cat,
                    )
                })
                .collect(),
        },
    )
    .placeholder("Select a command for more information ⌘");

    let owner = UserId::new(info.owner.id).to_user(&ctx).await?;

    let mut embed = CreateEmbed::new()
        .title("Hello,")
        .description(&persona.summary)
        .thumbnail(&avatar_url)
        .color(persona.interactions.colors.primary)
        .author(CreateEmbedAuthor::new(author_name).icon_url(avatar_url));

    let mut footer = CreateEmbedFooter::new(
        info.signature_for(owner.global_name.as_deref().unwrap_or(&owner.name))
            .unwrap_or_default(),
    );

    if let Some(icon_url) = &info.owner.icon_url {
        footer = footer.icon_url(icon_url);
    };

    embed = embed.footer(footer);

    let reply = ctx
        .send(
            CreateReply::default()
                .embed(embed)
                .components(vec![CreateActionRow::SelectMenu(select_menu)]),
        )
        .await?;

    let message = reply.into_message().await?;

    let mut interaction_stream = ComponentInteractionCollector::new(serenity_context)
        .message_id(message.id)
        .timeout(Duration::from_secs(30))
        .stream();

    while let Some(interaction) = interaction_stream.next().await {
        let selected_cat = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values.first(),
            _ => None,
        }
        .ok_or("invalid categoery")?;

        let cat_commands = by_category
            .get(selected_cat.as_str())
            .ok_or("unknown category")?;

        let prefix = &ctx_data.config.info.prefix;

        //TODO could use a clean up
        interaction
            .create_response(
                serenity_context,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(
                            CreateEmbed::new()
                                .title(format!(
                                    "{} - {} {} Command ⌘\n\n*Command List*:\n-----------------",
                                    cat_commands.len(),
                                    cat_emoji(selected_cat),
                                    capitalize(selected_cat)
                                ))
                                .description(
                                    cat_commands
                                        .iter()
                                        .map(|cmd| {
                                            let name: &str = &cmd.name;
                                            let usage = std::iter::once(name)
                                                .chain(
                                                    cmd.aliases
                                                        .iter()
                                                        .map(|alias| alias.as_ref())
                                                        .filter(|alias| *alias != name),
                                                )
                                                .map(|key| format!("{prefix}{key}"))
                                                .collect::<Vec<_>>()
                                                .join(", ");

                                            format!(
                                                "**{}**: *{}*\nusage: `{usage} or /{name}`\n",
                                                capitalize(name),
                                                cmd.description
                                                    .as_deref()
                                                    .unwrap_or("No description")
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )
                                .colour(persona.interactions.colors.accent),
                        )
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    message.delete(ctx).await?;
    Ok(())
}
