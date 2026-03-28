use serde::Deserialize;

pub mod characters;
pub mod commands;
pub mod info;
pub mod interactions;

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    pub characters_registry: characters::CharacterRegistry,
    pub commands_registry: commands::CommandRegistry,
    pub info_registry: info::InfoRegistry,
    pub interactions_registry: interactions::InteractionsRegistry,
}

#[derive(Debug, Clone)]
pub struct Env {}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
