use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct CommandChoice {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub r#type: u8, // r# reserved name
    pub required: bool,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub choices: Option<Vec<CommandChoice>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandConfig {
    pub name: String,
    pub keys: Vec<String>,
    pub category: String,
    pub description: String,
    pub options: Option<Vec<CommandOption>>,
}

#[derive(Deserialize, Debug)]
pub struct CommandRegistry {
    #[serde(flatten)]
    pub commands: HashMap<String, CommandConfig>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let json_data = include_str!("../../config/commands.json");
        serde_json::from_str(json_data).expect("Critical Error: commands.json is malformed!")
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&CommandConfig> {
        self.commands.get(name)
    }
}
