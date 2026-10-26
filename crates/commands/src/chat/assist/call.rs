use genai::chat::Tool as AiTool;

use super::{lookup::LookUp, tool::Tool};

/// something gemini can call: a read-only lookup or one of your commands
pub(super) enum Call<'a> {
    Lookup(LookUp),
    Command(&'a Tool<'a>),
}

impl<'a> Call<'a> {
    /// what the ai sees: the lookups first, then your commands
    pub fn declarations(commands: &[Tool]) -> Vec<AiTool> {
        let mut all = LookUp::declarations();
        all.extend(commands.iter().map(Tool::declaration));
        all
    }

    /// finds what gemini asked for by name, None when it made one up
    pub fn find(name: &str, commands: &'a [Tool<'a>]) -> Option<Self> {
        if let Ok(lookup) = name.parse() {
            return Some(Self::Lookup(lookup));
        }

        commands
            .iter()
            .find(|tool| tool.command.name == name)
            .map(Self::Command)
    }
}
