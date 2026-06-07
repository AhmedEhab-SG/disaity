use gemini_rust::Gemini;
use poise::Context as BaseContext;
use reqwest::Client;
use serenity::async_trait;

use crate::{
    commands::subscription::Subscription,
    config::Config,
    core::{errors::Error, utils::Utils},
};

pub type Context<'a> = BaseContext<'a, Data, Error>;

pub struct Data {
    pub http: Client,
    pub agent: Gemini,
    pub config: Config,
    pub subscription: Subscription,
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
        let http = Client::new();
        let subscription = Subscription::new();

        Self {
            http,
            agent,
            config,
            subscription,
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }
}
