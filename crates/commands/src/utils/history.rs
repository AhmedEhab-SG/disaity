use serenity::all::{GetMessages, MessageInteractionMetadata};

use disaity_config::CommandId;
use disaity_core::{Context, Error};

use super::Utils;

/// who said a turn: Client is this bot, User is whoever ran the command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakerRole {
    Client,
    User,
}

pub struct History {
    pub command_id: CommandId,
    pub lookback: u8,
    pub max_turns: usize,
    pub conversation: Vec<(SpeakerRole, String)>,
}

/// for the ai chat commands, ask and assist
impl Utils<'_> {
    const MAX_MSG: usize = 1900;
    const MAX_QUOTED_MSG: usize = 300;
    const QUOTE_PREFIX: &'static str = ">";

    fn quote_query(query: &str) -> String {
        let mut max_query = query.chars().take(Self::MAX_QUOTED_MSG).collect::<String>();

        if max_query.chars().count() == Self::MAX_QUOTED_MSG {
            max_query.push('…');
        }

        let quoted = max_query
            .lines()
            .map(|line| format!("{} {line}", Self::QUOTE_PREFIX))
            .collect::<Vec<_>>()
            .join("\n");

        format!("{quoted}\n\n")
    }

    pub fn res_with_first_quoted(&self, response: String, query: String) -> (String, Vec<String>) {
        let quoted_query = match self.ctx {
            Context::Application(_) => Self::quote_query(query.trim()),
            _ => String::new(),
        };

        let res_text_chars = response.chars().collect::<Vec<char>>();

        // discord can only handle limited text in msg makes chunks;
        // the 1st one also carries the quoted prompt
        let first_chunk_len = Self::MAX_MSG
            .saturating_sub(quoted_query.chars().count())
            .min(res_text_chars.len());

        let (first, rest) = res_text_chars.split_at(first_chunk_len);

        // quoted on only the 1st message
        let first_chunk = format!("{quoted_query}{}", first.iter().collect::<String>());

        let chunks = rest
            .chunks(Self::MAX_MSG)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>();

        (first_chunk, chunks)
    }

    /// Get earlier commands request and response, oldest first
    /// `lookback` channel messages are searched, at most `max_turns` turns are kept
    pub async fn get_cmd_history(
        &self,
        command_id: &CommandId,
        lookback: u8,
        max_turns: usize,
    ) -> Result<History, Error> {
        let config = &self.ctx.data().config;
        let prefix = &config.info.prefix;
        let keys = &config.commands_registry.get_command(command_id).keys;

        let messages = self
            .ctx
            .channel_id()
            .messages(self.ctx.http(), GetMessages::new().limit(lookback))
            .await?;

        let current_id = match self.ctx {
            Context::Prefix(p) => p.msg.id.get(),
            Context::Application(p) => p.interaction.id.get(),
        };

        let user_id = self.ctx.author().id;
        let bot_id = self.ctx.framework().bot_id();

        // "~assist kick bob" -> "kick bob", None for anything else
        let asked = |content: &str| {
            keys.iter()
                .find_map(|k| content.strip_prefix(&format!("{prefix}{k} ")))
                .map(|rest| rest.trim().to_owned())
        };

        let mut history = messages
            .into_iter()
            .filter_map(|m| {
                // Ignore the current command trigger
                if m.id.get() == current_id {
                    return None;
                }

                // say! and reply! send an embed on slash, plain text on prefix
                let content = match m.embeds.first() {
                    Some(embed) if m.content.is_empty() => embed.description.clone()?,
                    _ => m.content.clone(),
                };

                if content.is_empty() {
                    return None;
                }

                let is_bot = m.author.id == bot_id;

                // Handle User Prefix Messages
                if !is_bot {
                    if m.author.id != user_id {
                        return None;
                    }

                    return Some(vec![(SpeakerRole::User, asked(&content)?)]);
                }

                //  Handle Bot Messages (Prefix replies & Slash command responses)
                let is_reply_to_user = m.referenced_message.as_ref().is_some_and(|ref_msg| {
                    ref_msg.author.id == user_id && asked(&ref_msg.content).is_some()
                });

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
                        && let Some(lines) = prompt_block
                            .lines()
                            .map(|line| {
                                line.strip_prefix(&format!("{} ", Self::QUOTE_PREFIX))
                                    .or(line.strip_prefix(Self::QUOTE_PREFIX))
                            })
                            .collect::<Option<Vec<_>>>()
                    {
                        let user_prompt = lines.join("\n").trim().to_string();

                        let bot_response = response.trim().to_string();

                        // Yield two messages from one Discord message
                        return Some(vec![
                            (SpeakerRole::Client, bot_response),
                            (SpeakerRole::User, user_prompt),
                        ]);
                    }

                    // then its other command
                    return None;
                }

                // Regular prefix command reply
                Some(vec![(SpeakerRole::Client, content.trim().to_string())])
            })
            .flatten()
            .take(max_turns)
            .collect::<Vec<(SpeakerRole, String)>>();

        history.reverse();

        // the cap or limit can cut the question, start on a question and removes orphen answer
        let first_question = history
            .iter()
            .position(|(role, _)| *role == SpeakerRole::User);

        history.drain(..first_question.unwrap_or(history.len()));

        Ok(History {
            command_id: command_id.to_owned(),
            lookback,
            max_turns,
            conversation: history,
        })
    }
}
