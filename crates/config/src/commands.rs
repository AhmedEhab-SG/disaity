use std::{collections::HashMap, fmt::Debug, fs, path::Path, str::FromStr};

use serde::Deserialize;
use strum::{Display, EnumString, VariantArray};

use super::utils::{ConfigError, Merge};

#[derive(Display, Deserialize, Debug, Clone, Eq, Hash, PartialEq, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CommandId {
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

#[derive(Display, Deserialize, Debug, Clone, Eq, Hash, PartialEq, EnumString, VariantArray)]
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
pub struct CommandEntry {
    pub name: String,
    pub keys: Vec<String>,
    pub category: String,
    pub description: String,
    pub timeout: Option<u64>,
    pub action: Option<String>,
    pub options: Option<Vec<CommandOption>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandRegistry {
    #[serde(flatten)]
    pub categories: HashMap<Category, Vec<CommandEntry>>,

    #[serde(flatten)]
    pub commands: HashMap<CommandId, CommandEntry>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct CommandDetailsOverride {
    pub keys: Option<Vec<String>>,
    pub description: Option<String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let json_data = include_str!("../default/commands.json");
        let commands: HashMap<CommandId, CommandEntry> =
            serde_json::from_str(json_data).expect("Critical Error: commands.json is malformed!");

        let mut categories: HashMap<Category, Vec<CommandEntry>> = HashMap::new();

        for config in commands.values() {
            if let Ok(category_enum) = Category::from_str(&config.category) {
                categories
                    .entry(category_enum)
                    .or_default()
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

impl Merge for CommandRegistry {}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_command(&self, cmd: &CommandId) -> &CommandEntry {
        self.commands.get(cmd).unwrap_or_else(|| {
            panic!("Expected command '{cmd}', but it was not found in registry",);
        })
    }

    pub fn get_cmds_from_cat(&self, cat: &Category) -> &Vec<CommandEntry> {
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

    pub fn from_file_over(mut self, path: &Path) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path)?;
        let overwrite: HashMap<String, CommandDetailsOverride> = serde_json::from_str(&raw)?;

        for (name, over) in overwrite {
            let Ok(cmd) = CommandId::from_str(&name) else {
                eprintln!("Warning: unknown command '{name}' in {}", path.display());
                continue;
            };

            let Some(config) = self.commands.get_mut(&cmd) else {
                continue;
            };

            if let Some(keys) = over.keys {
                config.keys = keys
            }

            if let Some(description) = over.description {
                config.description = description;
            }
        }

        Ok(self)
    }
}
