use genai::chat::{ChatMessage, ChatResponse, MessageContent, StopReason};

use disaity_config::Persona;
use disaity_core::AiAgent;

use super::{
    Utils,
    history::{History, SpeakerRole},
};

impl History {
    pub fn into_turns(self, request: Option<impl Into<MessageContent>>) -> Vec<ChatMessage> {
        let mut turns = self
            .conversation
            .into_iter()
            .map(|(role, content)| match role {
                SpeakerRole::Client => ChatMessage::assistant(content),
                SpeakerRole::User => ChatMessage::user(content),
            })
            .collect::<Vec<ChatMessage>>();

        if let Some(content) = request {
            turns.push(ChatMessage::user(content));
        }

        turns
    }
}

impl Utils<'_> {
    /// the ai's last words, or a persona line for why it has none
    /// `acted` is true when the ai already ran something for this request
    pub fn ai_answer(res: ChatResponse, acted: bool, persona: &Persona) -> String {
        let stop = res.stop_reason.clone();

        let text = AiAgent::text(res);

        if !text.trim().is_empty() {
            return text;
        }

        let ai_persona = &persona.interactions.events.ai;

        let lines = match stop {
            Some(StopReason::ContentFilter(_)) => &ai_persona.on_content_filter,
            Some(StopReason::MaxTokens(_)) => &ai_persona.on_max_tokens,
            _ if acted => &ai_persona.on_done,
            _ => &ai_persona.on_no_answer,
        };

        lines.get_random_res().unwrap_or("…").to_owned()
    }
}
