use std::time::Duration;

use poise::CreateReply;
use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, Permissions,
};
use strum::{Display, EnumMessage, EnumString};

use disaity_core::{Context, Error, send};

#[derive(Display, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumString, EnumMessage)]
pub(super) enum Confirm {
    #[strum(to_string = "assist_confirm", message = "Confirm")]
    Yes,

    #[strum(to_string = "assist_cancel", message = "Cancel")]
    No,
}

impl Confirm {
    /// permissions that make a command worth a confirm click
    pub fn powerful() -> Permissions {
        Permissions::ADMINISTRATOR
            | Permissions::MANAGE_GUILD
            | Permissions::MANAGE_ROLES
            | Permissions::MANAGE_CHANNELS
            | Permissions::MANAGE_MESSAGES
            | Permissions::MANAGE_NICKNAMES
            | Permissions::KICK_MEMBERS
            | Permissions::BAN_MEMBERS
            | Permissions::MODERATE_MEMBERS
            | Permissions::MOVE_MEMBERS
    }

    /// show the commands and action that about to run and ask for confirmation
    pub async fn confirm(ctx: Context<'_>, lines: &[String]) -> Result<bool, Error> {
        let colors = &ctx.data().config.persona.interactions.colors;

        let embed = CreateEmbed::new()
            .title("I'm about to run:")
            .description(
                lines
                    .iter()
                    .map(|line| format!("• {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .color(colors.warning);

        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(Self::Yes.to_string())
                .label(Self::Yes.get_message().unwrap_or_default())
                .style(ButtonStyle::Danger),
            CreateButton::new(Self::No.to_string())
                .label(Self::No.get_message().unwrap_or_default())
                .style(ButtonStyle::Secondary),
        ]);

        let reply = send!(
            ctx,
            CreateReply::default()
                .embed(embed.clone())
                .components(vec![buttons])
        );

        let msg_id = reply.message().await?.id;

        // only the requester can answer, other people's clicks are ignored
        let press = ComponentInteractionCollector::new(ctx.serenity_context())
            .message_id(msg_id)
            .author_id(ctx.author().id)
            .timeout(Duration::from_secs(30))
            .await;

        let approved = press
            .as_ref()
            .is_some_and(|p| p.data.custom_id == Self::Yes.to_string());

        if let Some(press) = &press {
            press
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;
        }

        let status = match (&press, approved) {
            (None, _) => "timed out, nothing was run",
            (_, true) => "confirmed",
            (_, false) => "cancelled, discard actions",
        };

        // drop the buttons so the plan can't be approved twice
        reply
            .edit(
                ctx,
                CreateReply::default()
                    .embed(embed.footer(CreateEmbedFooter::new(status)))
                    .components(Vec::new()),
            )
            .await?;

        Ok(approved)
    }
}
