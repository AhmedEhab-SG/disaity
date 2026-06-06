use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use strum::{Display, EnumString};

#[derive(Display, Deserialize, Debug, Clone, Eq, Hash, PartialEq, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Command {
    Ask,
    Help,
    Join,
    Leave,
    Clear,
    Jump,
    Pause,
    Play,
    Queue,
    Repeat,
    Resume,
    Seek,
    Shuffle,
    Skip,
    Stop,
    Volume,
    Prayer,
    ClearPrayer,
}

#[derive(Display, Deserialize, Debug, Clone, Eq, Hash, PartialEq, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Category {
    Chat,
    Music,
    Subscription,
    Other,
}

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
    pub timeout: Option<u64>,
    pub action: Option<String>,
    pub options: Option<Vec<CommandOption>>,
}

#[derive(Deserialize, Debug)]
pub struct CommandRegistry {
    #[serde(flatten)]
    pub categories: HashMap<Category, Vec<CommandConfig>>,

    #[serde(flatten)]
    pub commands: HashMap<Command, CommandConfig>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let json_data = include_str!("../../config/commands.json");
        let commands: HashMap<Command, CommandConfig> =
            serde_json::from_str(json_data).expect("Critical Error: commands.json is malformed!");

        let mut categories: HashMap<Category, Vec<CommandConfig>> = HashMap::new();

        for config in commands.values() {
            if let Ok(category_enum) = Category::from_str(&config.category) {
                categories
                    .entry(category_enum)
                    .or_insert_with(Vec::new)
                    .push(config.clone());
            } else {
                eprintln!(
                    "Warning: Unknown category '{}' in command '{}'",
                    config.category, config.name
                );
            }
        }

        Self {
            categories,
            commands,
        }
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_command(&self, cmd: &Command) -> &CommandConfig {
        self.commands.get(cmd).unwrap_or_else(|| {
            panic!("Expected command '{cmd}', but it was not found in registry",);
        })
    }

    pub fn get_cmds_from_cat(&self, cat: &Category) -> &Vec<CommandConfig> {
        self.categories
            .get(cat)
            .unwrap_or_else(|| panic!("Expect category '{cat}', but it wasnt found in registry"))
    }

    pub fn get_cat_emoji(&self, cat: &Category) -> &str {
        match cat {
            Category::Music => "🎶",
            Category::Subscription => "📢",
            Category::Chat => "🗨️",
            Category::Other => "⚙️",
        }
    }
}
