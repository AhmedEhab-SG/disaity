use genai::chat::{ChatMessage, ChatRequest};
use poise::{Context as MessageContext, command};

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

    let history = Utils::get_cmd_history(&ctx, &data.config, &CommandId::Ask, 50).await?;

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

    // discord can only handle limited text in msg makes chunks
    let char_vec: Vec<char> = res_text.chars().collect();
    let chunks = char_vec.chunks(1900);

    for (i, chunk) in chunks.enumerate() {
        let mut content: String = chunk.iter().collect();

        //  reply on only the 1st message
        if i == 0 {
            if let MessageContext::Application(_) = ctx {
                content = format!("> {}\n\n{}", chat.trim(), content);
            }
            ctx.reply(content).await?;
            continue;
        }

        ctx.say(content).await?;
    }

    Ok(())
}
