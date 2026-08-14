mod assets;
mod commands;
mod env;
mod info;
mod persona;
mod utils;

pub use assets::{Assets, Provider};
pub use commands::{Category, Command, CommandRegistry};
use env::Env;
use info::Info;
pub use persona::{Persona, Preset};

#[derive(Default, Debug)]
pub struct Config {
    pub persona: Persona,
    pub commands_registry: CommandRegistry,
    pub info: Info,
    pub assets: Assets,
    pub env: Env,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct ConfigBuilder {
    pub persona: Option<Persona>,
    pub info: Option<Info>,
}
