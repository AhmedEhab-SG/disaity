pub mod characters;
pub mod commands;
pub mod env;
pub mod info;
pub mod interactions;

#[derive(Default, Debug)]
pub struct Config {
    pub characters_registry: characters::CharacterRegistry,
    pub commands_registry: commands::CommandRegistry,
    pub info_registry: info::InfoRegistry,
    pub interactions_registry: interactions::InteractionsRegistry,
    pub env: env::Env,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
