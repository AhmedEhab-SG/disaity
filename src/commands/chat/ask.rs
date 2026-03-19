use poise::{Context as MessageContext, command};
use serenity::all::GetMessages;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "ask", aliases("a"))]
pub async fn ask(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    ctx.defer().await?;
    ctx.channel_id()
        .broadcast_typing(&ctx.serenity_context().http)
        .await
        .ok();

    let channel_id = ctx.channel_id();
    let http = &ctx.serenity_context().http;
    let agent = &ctx.data().agent;
    let user_id = ctx.author().id;

    let messages = channel_id
        .messages(http, GetMessages::new().limit(50))
        .await?;

    let current_msg_id = match ctx {
        MessageContext::Prefix(p) => Some(p.msg.id),
        MessageContext::Application(_) => None,
    };

    let history: Vec<(bool, String)> = messages
        .into_iter()
        .filter(|m| {
            if Some(m.id) == current_msg_id || m.content.is_empty() {
                return false;
            }

            if !m.author.bot
                && !(m.content.as_str().starts_with("-ask ")
                    || m.content.as_str().starts_with("-a ")
                    || m.content.as_str().starts_with("/ask "))
            {
                return false;
            }

            let is_target_user = m.author.id == user_id;

            let is_bot_replying_to_user = m.author.bot
                && m.referenced_message
                    .as_ref()
                    .is_some_and(|ref_msg| ref_msg.author.id == user_id);

            is_target_user || is_bot_replying_to_user
        })
        .map(|m| {
            let mut clean_content = m.content.as_str();
            let is_bot = m.author.bot;

            if !is_bot {
                // Clean out prefix commands
                if clean_content.starts_with("-ask ") {
                    clean_content = &clean_content[5..];
                } else if clean_content.starts_with("-a ") {
                    clean_content = &clean_content[3..];
                } else if clean_content.starts_with("/ask ") {
                    clean_content = &clean_content[5..];
                }
            }

            (is_bot, clean_content.trim().to_string())
        })
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let mut req = agent.generate_content();

    for (is_bot, content) in history.clone() {
        if is_bot {
            req = req.with_model_message(content);
        } else {
            req = req.with_user_message(content);
        }
    }

    req = req.with_user_message(msg.trim());

    let res_text = req.execute().await?.text();

    let char_vec: Vec<char> = res_text.chars().collect();
    let chunks = char_vec.chunks(1900);

    for (i, chunk) in chunks.enumerate() {
        let content: String = chunk.iter().collect();

        if i == 0 {
            ctx.reply(content).await?;
        } else {
            channel_id.say(http, content).await?;
        }
    }

    Ok(())
}
