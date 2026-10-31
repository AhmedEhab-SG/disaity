use std::time::Duration;

use poise::Context as MessageContext;
use serenity::all::{GetMessages, MessageInteractionMetadata};

use disaity_config::{CommandId, Config};
use disaity_core::{Context, Error, Transcript};

pub struct Utils<'a> {
    ctx: Context<'a>,
}

impl<'a> Utils<'a> {
    pub const MAX_MSG: usize = 1900;
    pub const MAX_QUOTED_MSG: usize = 300;
    pub const QUOTE_PREFIX: &'static str = ">";

    pub fn new(ctx: Context<'a>) -> Self {
        Self { ctx }
    }

    pub fn quote_query(query: &str) -> String {
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
            MessageContext::Application(_) => Self::quote_query(query.trim()),
            _ => String::new(),
        };

        // discord can only handle limited text in msg makes chunks
        let res_text_chars = response.chars().collect::<Vec<char>>();

        // discord can only handle limited text in msg makes chunks;
        // the 1st one also carries the quoted prompt
        let first_chunk_len = Utils::MAX_MSG
            .saturating_sub(quoted_query.chars().count())
            .min(res_text_chars.len());

        let (first, rest) = res_text_chars.split_at(first_chunk_len);

        // quoted on only the 1st message
        let first_chunk = format!("{quoted_query}{}", first.iter().collect::<String>());

        let chunks = rest
            .chunks(Utils::MAX_MSG)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>();

        (first_chunk, chunks)
    }

    /// get chat history and filter a conversations between user and bot with prefix
    pub async fn get_cmd_history(
        &self,
        config: &Config,
        command_id: &CommandId,
        limit: u8,
    ) -> Result<Vec<(bool, String)>, Error> {
        let prefix = &config.info.prefix;
        let keys = &config.commands_registry.get_command(command_id).keys;

        let messages = self
            .ctx
            .channel_id()
            .messages(self.ctx.http(), GetMessages::new().limit(limit))
            .await?;

        let current_msg_id: Option<u64> = match self.ctx {
            MessageContext::Prefix(p) => Some(p.msg.id.into()),
            MessageContext::Application(p) => Some(p.interaction.id.into()),
        };

        let user_id = self.ctx.author().id;

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

        Ok(history)
    }

    pub async fn audit_reason(ctx: Context<'_>, reason: &str) -> String {
        let authur = ctx.author();

        let via = match ctx.invocation_data::<Transcript>().await {
            Some(_) => format!(" via {}", ctx.data().config.persona.name),
            None => String::new(),
        };

        // disord limits log to 512 char
        format!("{} ({}){via}: {reason}", authur.name, authur.id)
            .chars()
            .take(512)
            .collect()
    }

    /// Parse user-supplied timestamp into `Duration`.
    /// Accepts:
    /// - "MM:SS" (e.g. "1:20")
    /// - "HH:MM:SS" (e.g. "1:02:30")
    /// - "Xs", "Xm", "Xh", or combined like "1h2m3s"
    /// - plain integer "90" interpreted as seconds
    pub fn parse_timestamp(input: &str) -> Result<Duration, String> {
        let s = input.trim().to_lowercase();

        if s.contains(':') {
            let parts = s.split(':').count();
            return s
                .split(':')
                .try_fold(0u64, |acc, p| {
                    acc.checked_mul(60)?.checked_add(p.trim().parse().ok()?)
                })
                .filter(|_| matches!(parts, 2 | 3))
                .map(Duration::from_secs)
                .ok_or_else(|| "Use MM:SS or HH:MM:SS".to_owned());
        }

        if let Ok(secs) = s.parse() {
            return Ok(Duration::from_secs(secs));
        }

        s.parse::<jiff::SignedDuration>()
            .ok()
            .and_then(|d| Duration::try_from(d).ok())
            .ok_or_else(|| "Unrecognised format".to_owned())
    }

    pub fn format_duration_human(d: Duration) -> String {
        let secs = d.as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{}:{:02}", mins, secs)
        }
    }
}
