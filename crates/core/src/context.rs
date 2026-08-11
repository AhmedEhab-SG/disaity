use gemini_rust::{Gemini, Model};
use poise::Context as BaseContext;
use reqwest::Client;
use serenity::async_trait;

use disaity_config::Config;

use crate::{errors::Error, state::Subscription, utils::Utils};

pub type Context<'a> = BaseContext<'a, Data, Error>;

pub struct AiAgent {
    pub agent: Gemini,
    pub fallback_agent: Gemini,
}

pub struct Data {
    pub http: Client,
    pub ai: AiAgent,
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

impl Data {
    pub async fn new() -> Result<Self, Error> {
        let config = Config::new();
        let agent = Gemini::new(&config.env.gemini_api_key).expect("failed to connect to gemini");
        let fallback_agent =
            Gemini::with_model(&config.env.gemini_api_key, Model::Gemini25FlashLite)
                .expect("failed to connect to gemini");
        let http = Client::new();
        let subscription = Subscription::connect(&config.env.db_path).await?;

        Ok(Self {
            http,
            ai: AiAgent {
                agent,
                fallback_agent,
            },
            config,
            subscription,
        })
    }
}

#[derive(Default)]
pub struct DataBuilder {
    config: Option<Config>,
    http: Option<Client>,
}

impl DataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_http(mut self, http: Client) -> Self {
        self.http = Some(http);
        self
    }

    pub async fn build(self) -> Result<Data, Error> {
        let config = self.config.unwrap_or_default();
        let agent = Gemini::new(&config.env.gemini_api_key).expect("failed to connect to gemini");
        let fallback_agent =
            Gemini::with_model(&config.env.gemini_api_key, Model::Gemini25FlashLite)
                .expect("failed to connect to gemini");

        Ok(Data {
            http: self.http.unwrap_or_default(),
            ai: AiAgent {
                agent,
                fallback_agent,
            },
            subscription: Subscription::connect(&config.env.db_path).await?,
            config,
        })
    }
}
