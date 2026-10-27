use std::time::Duration;

use poise::Context as MessageContext;
use serenity::all::{GetMessages, MessageInteractionMetadata};

use disaity_config::{CommandId, Config};
use disaity_core::{Context, Error, Transcript};

pub struct Utils;

impl Utils {
    /// get chat history and filter a conversations between user and bot with prefix
    pub async fn get_cmd_history(
        ctx: &Context<'_>,
        config: &Config,
        command_id: &CommandId,
        limit: u8,
    ) -> Result<Vec<(bool, String)>, Error> {
        let prefix = &config.info.prefix;
        let keys = &config.commands_registry.get_command(command_id).keys;

        let messages = ctx
            .channel_id()
            .messages(ctx.http(), GetMessages::new().limit(limit))
            .await?;

        let current_msg_id: Option<u64> = match ctx {
            MessageContext::Prefix(p) => Some(p.msg.id.into()),
            MessageContext::Application(p) => Some(p.interaction.id.into()),
        };

        let user_id = ctx.author().id;

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
        if s.is_empty() {
            return Err("Empty time string".to_owned());
        }

        // Case 1: colon-separated (HH:MM:SS or MM:SS)
        if s.contains(':') {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 2 || parts.len() == 3 {
                // parse from right: seconds, minutes, optional hours
                let try_parse = |p: &str| -> Result<u64, String> {
                    p.trim()
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid number in time part: `{}`", p))
                };

                let secs = try_parse(parts[parts.len() - 1])?;
                let mins = try_parse(parts[parts.len() - 2])?;
                let hours = if parts.len() == 3 {
                    try_parse(parts[0])?
                } else {
                    0
                };

                // validate ranges a bit (optional)
                if mins >= 60 || secs >= 60 {
                    // It's common to accept >=60, but warn user: we choose to reject for clarity.
                    return Err("Minutes/seconds must be less than 60 in colon format (use `90s` for 90 seconds).".to_owned());
                }

                let total_secs = hours * 3600 + mins * 60 + secs;
                return Ok(Duration::from_secs(total_secs));
            } else {
                return Err("Unsupported colon format. Use MM:SS or HH:MM:SS.".to_owned());
            }
        }

        // Case 2: suffix style like 1h2m3s, or plain integer seconds
        // We'll walk the string collecting digits then mapping following letter to unit.
        let mut total_secs: u64 = 0;
        let mut num_buf = String::new();
        let mut found_unit = false;

        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch.is_ascii_digit() {
                num_buf.push(ch);
                // continue until a non-digit
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        num_buf.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // after a number there might be a unit char (h/m/s) or string ends -> seconds
                if let Some(&next) = chars.peek() {
                    match next {
                        'h' | 'm' | 's' => {
                            let val = num_buf
                                .parse::<u64>()
                                .map_err(|_| format!("Invalid number: {}", num_buf))?;
                            match next {
                                'h' => {
                                    total_secs = total_secs
                                        .checked_add(val.checked_mul(3600).ok_or("overflow")?)
                                        .ok_or("overflow")?
                                }
                                'm' => {
                                    total_secs = total_secs
                                        .checked_add(val.checked_mul(60).ok_or("overflow")?)
                                        .ok_or("overflow")?
                                }
                                's' => {
                                    total_secs = total_secs.checked_add(val).ok_or("overflow")?
                                }
                                _ => {}
                            }
                            found_unit = true;
                            num_buf.clear();
                            chars.next(); // consume unit char
                        }
                        // If next is not a unit, treat the number as seconds and continue (e.g. "90")
                        _ => {
                            let val = num_buf
                                .parse::<u64>()
                                .map_err(|_| format!("Invalid number: {}", num_buf))?;
                            total_secs = total_secs.checked_add(val).ok_or("overflow")?;
                            num_buf.clear();
                        }
                    }
                } else {
                    // end of string: interpret number as seconds
                    let val = num_buf
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid number: {}", num_buf))?;
                    total_secs = total_secs.checked_add(val).ok_or("overflow")?;
                    num_buf.clear();
                }
            } else if ch.is_whitespace() {
                continue;
            } else {
                return Err(format!("Unexpected character `{}` in time string", ch));
            }
        }

        if total_secs == 0 && !found_unit {
            // could be that user typed "0" or invalid
            if s == "0" || s == "0s" {
                return Ok(Duration::from_secs(0));
            } else {
                return Err("Couldn't parse time. Try `1:20`, `80s`, or `1m20s`.".to_owned());
            }
        }

        Ok(Duration::from_secs(total_secs))
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
