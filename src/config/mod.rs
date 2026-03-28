use crate::config::{characters::CharacterRegistry, commands::CommandRegistry, info::InfoRegistry};

pub mod characters;
pub mod commands;
pub mod info;
pub mod interactions;

pub struct Config {
    pub characters_registry: CharacterRegistry,
    pub commands_registry: CommandRegistry,
    pub info_registry: InfoRegistry,
    pub interactions_registry: InfoRegistry,
}
