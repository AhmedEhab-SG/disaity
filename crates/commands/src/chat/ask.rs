use genai::chat::ChatRequest;
use poise::command;

use disaity_config::CommandId;
use disaity_core::{Context, Error, reply, say};

use super::super::{Check, Utils};

/// using ai to chat with presona
#[command(
    slash_command,
    prefix_command,
    user_cooldown = 15,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | READ_MESSAGE_HISTORY",
    check = "Check::dm_with_auth"
)]
pub async fn ask(
    ctx: Context<'_>,

    #[description = "Take to me."]
    #[rest]
    chat: String,
) -> Result<(), Error> {
    let data = &ctx.data();

    let ai = data
        .ai
        .as_ref()
        .ok_or("AI is not configured on this bot.")?;

    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let cmd_utils = Utils::new(ctx);

    let turns = cmd_utils
        .get_cmd_history(&CommandId::Ask, 50, 10)
        .await?
        .into_turns(Some(chat.trim()));

    let req = ChatRequest::new(turns).with_system(&data.config.persona.personality.system);

    let res_text = Utils::ai_answer(ai.chat(&req).await?, false, &data.config.persona);

    let (first, rest) = cmd_utils.res_with_first_quoted(res_text, chat);

    drop(typing);

    reply!(ctx, first);

    for chunk in rest {
        say!(ctx, chunk);
    }

    Ok(())
}
