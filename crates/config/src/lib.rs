mod assets;
mod commands;
mod env;
mod info;
mod persona;
mod utils;

pub use assets::{Assets, Provider};
pub use commands::{Category, CommandId, CommandRegistry};
pub use env::{Env, EnvBuilder, LogLevel};
pub use info::Info;
pub use persona::{ActivityType, Persona, Preset, Status};

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

#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    pub persona: Option<Persona>,
    pub info: Option<Info>,
    pub env: Option<Env>,
    pub commands_registry: Option<CommandRegistry>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_persona(mut self, persona: Persona) -> Self {
        self.persona = Some(persona);
        self
    }

    pub fn with_info(mut self, info: Info) -> Self {
        self.info = Some(info);
        self
    }

    pub fn with_env(mut self, env: Env) -> Self {
        self.env = Some(env);
        self
    }

    pub fn with_commands_registry(mut self, registry: CommandRegistry) -> Self {
        self.commands_registry = Some(registry);
        self
    }

    pub fn build(self) -> Config {
        Config {
            persona: self.persona.unwrap_or_default(),
            info: self.info.unwrap_or_default(),
            env: self.env.unwrap_or_default(),
            commands_registry: self.commands_registry.unwrap_or_default(),
            ..Default::default()
        }
    }
}
