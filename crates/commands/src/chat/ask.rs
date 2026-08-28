use gemini_rust::{Gemini, InteractionContent, InteractionThinkingLevel, Step};
use poise::{Context as MessageContext, command};
use serenity::all::{GetMessages, Message, MessageInteractionMetadata, UserId};

use crate::checks::dm_with_auth;
use disaity_config::{CommandId, Config};
use disaity_core::{Context, Error};

fn get_history(
    messages: Vec<Message>,
    current_msg_id: Option<u64>,
    user_id: UserId,
    config: &Config,
) -> Vec<(bool, String)> {
    let prefix = &config.info.prefix;
    let keys = &config.commands_registry.get_command(&CommandId::Ask).keys;

    let mut history: Vec<(bool, String)> = messages
        .into_iter()
        .filter_map(|m| {
            // Ignore the current command trigger and empty messages
            if Some(m.id.get()) == current_msg_id || m.content.is_empty() {
                return None;
            }

            let is_bot = m.author.bot;
            let content = m.content.clone();

            // Handle User Prefeix Messages
            if !is_bot {
                if m.author.id != user_id {
                    return None;
                }

                if !&keys
                    .iter()
                    .any(|k| content.starts_with(&format!("{prefix}{k} ")))
                {
                    return None;
                }

                let clean = keys
                    .iter()
                    .find_map(|k| content.strip_prefix(&format!("{prefix}{k} ")))
                    .unwrap_or(&content);

                return Some(vec![(is_bot, clean.trim().to_string())]);
            }

            //  Handle Bot Messages (Prefix replies & Slash command responses)
            let is_reply_to_user = m
                .referenced_message
                .as_ref()
                .is_some_and(|ref_msg| ref_msg.author.id == user_id);

            let is_interaction_for_user = match m.interaction_metadata.as_deref() {
                Some(MessageInteractionMetadata::Command(p)) => p.user.id == user_id,
                _ => false,
            };

            if !(is_reply_to_user || is_interaction_for_user) {
                return None;
            }

            // SLASH COMMAND HISTORY EXTRACTION
            // If the bot replied to a slash command, it formatted it with "> prompt \n\n response"
            if is_interaction_for_user {
                if let Some((prompt_block, response)) = content.split_once("\n\n")
                    && prompt_block.starts_with("> ")
                {
                    let user_prompt = prompt_block[2..].trim().to_string();
                    let bot_response = response.trim().to_string();

                    // Yield two messages from one Discord message
                    return Some(vec![(is_bot, bot_response), (!is_bot, user_prompt)]);
                }

                // then its other command
                return None;
            }

            // Regular prefix command reply
            Some(vec![(is_bot, content.trim().to_string())])
        })
        .flatten()
        .take(10)
        .collect();

    history.reverse();
    history
}

#[command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | READ_MESSAGE_HISTORY",
    check = "dm_with_auth"
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

    let messages = ctx
        .channel_id()
        .messages(ctx.http(), GetMessages::new().limit(50))
        .await?;

    let current_msg_id: Option<u64> = match ctx {
        MessageContext::Prefix(p) => Some(p.msg.id.into()),
        MessageContext::Application(p) => Some(p.interaction.id.into()),
    };

    let history = get_history(messages, current_msg_id, ctx.author().id, &data.config);
    let persona = &data.config.persona;

    // could use a clean up
    let build = |agent: &Gemini, history: &Vec<(bool, String)>| {
        let mut steps: Vec<Step> = history
            .iter()
            .map(|(is_bot, content)| {
                if *is_bot {
                    Step::ModelOutput {
                        content: vec![InteractionContent::text(content.as_str())],
                        error: None,
                    }
                } else {
                    Step::UserInput {
                        content: vec![InteractionContent::text(content.as_str())],
                    }
                }
            })
            .collect();

        steps.push(Step::UserInput {
            content: vec![InteractionContent::text(chat.trim())],
        });

        agent
            .create_interaction()
            .with_thinking_level(InteractionThinkingLevel::High)
            .with_google_search()
            .with_system_instruction(persona.personality.system.clone())
            .with_step_input(steps)
    };

    let res_text = match build(&ai.agent, &history).execute().await {
        Ok(res) => res.output_text(),
        Err(e) if e.to_string().contains("503") || e.to_string().contains("UNAVAILABLE") => {
            tracing::warn!(
                message = %persona.name,
                "Gemini primary model busy (503), falling back to flash-lite"
            );

            build(&ai.fallback_agent, &history)
                .execute()
                .await?
                .output_text()
        }
        Err(e_rest) => return Err(e_rest.into()),
    };

    let char_vec: Vec<char> = res_text.chars().collect();
    let chunks = char_vec.chunks(1900);

    for (i, chunk) in chunks.enumerate() {
        let mut content: String = chunk.iter().collect();

        if i == 0 {
            if let MessageContext::Application(_) = ctx {
                content = format!("> {}\n\n{}", chat.trim(), content);
            }
            ctx.reply(content).await?;
        } else {
            ctx.say(content).await?;
        }
    }

    Ok(())
}
