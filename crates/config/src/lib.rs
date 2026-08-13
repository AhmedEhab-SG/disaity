mod characters;
mod commands;
mod env;
mod info;
mod interactions;
mod persona;
mod utils;

pub use characters::{Character, CharacterRegistry};
pub use commands::{Category, Command, CommandRegistry};
use env::Env;
use info::InfoRegistry;
pub use interactions::{InteractionsRegistry, Provider};

#[derive(Default, Debug)]
pub struct Config {
    pub characters_registry: CharacterRegistry,
    pub commands_registry: CommandRegistry,
    pub info_registry: InfoRegistry,
    pub interactions_registry: InteractionsRegistry,
    pub env: Env,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
