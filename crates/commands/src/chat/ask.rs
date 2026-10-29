use genai::chat::{ChatMessage, ChatRequest};
use poise::command;

use disaity_config::CommandId;
use disaity_core::{AiAgent, Context, Error};

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

    let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    let cmd_utils = Utils::new(ctx);

    let history = cmd_utils
        .get_cmd_history(&data.config, &CommandId::Ask, 50)
        .await?;

    let persona = &data.config.persona;

    // build a conversations
    let mut turns = history
        .into_iter()
        .map(|(is_bot, content)| match is_bot {
            true => ChatMessage::assistant(content),
            false => ChatMessage::user(content),
        })
        .collect::<Vec<ChatMessage>>();

    turns.push(ChatMessage::user(chat.trim()));

    let req = ChatRequest::new(turns).with_system(persona.personality.system.clone());

    let res_text = AiAgent::text(ai.chat(&req).await?);

    let (first, rest) = cmd_utils.res_with_first_quoted(res_text, chat);

    ctx.reply(first).await?;

    for chunk in rest {
        ctx.say(chunk).await?;
    }

    Ok(())
}
