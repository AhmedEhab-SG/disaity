use gemini_rust::{Gemini, Model};
use poise::Context as BaseContext;
use reqwest::Client;
use serenity::async_trait;

use crate::{
    commands::CommandsRegistry,
    config::Config,
    core::{errors::Error, utils::Utils},
};

pub type Context<'a> = BaseContext<'a, Data, Error>;

pub struct AiAgent {
    pub agent: Gemini,
    pub fallback_agent: Gemini,
}

pub struct Data {
    pub http: Client,
    pub ai: AiAgent,
    pub config: Config,
    pub registry: CommandsRegistry,
}

#[async_trait]
pub trait ContextExt<'a> {
    fn utils(self) -> Utils<'a>;
}

#[async_trait]
impl<'a> ContextExt<'a> for Context<'a> {
    fn utils(self) -> Utils<'a> {
        Utils { ctx: self }
    }
}

impl Default for Data {
    fn default() -> Self {
        let config = Config::new();
        let agent = Gemini::new(&config.env.gemini_api_key).expect("failed to connect to gemini");
        let fallback_agent =
            Gemini::with_model(&config.env.gemini_api_key, Model::Gemini25FlashLite)
                .expect("failed to connect to gemini");
        let http = Client::new();
        let registry = CommandsRegistry::new(&config.commands_registry, &config.env.db_path);

        Self {
            http,
            ai: AiAgent {
                agent,
                fallback_agent,
            },
            config,
            registry,
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }
}
