use gemini_rust::{Gemini, Model};
use poise::Context as BaseContext;
use reqwest::Client;
use serenity::async_trait;

use disaity_config::Config;

use super::{db::Database, errors::Error, utils::Utils};

pub type Context<'ctx> = BaseContext<'ctx, Data, Error>;

pub struct AiAgent {
    pub agent: Gemini,
    pub fallback_agent: Gemini,
}

impl AiAgent {
    pub fn connect(key: &str) -> Result<Self, Error> {
        let agent = Gemini::new(key).map_err(|e| format!("failed to connect to gemini: {e}"))?;
        let fallback_agent = Gemini::with_model(key, Model::Gemini25FlashLite)
            .map_err(|e| format!("failed to connect to gemini: {e}"))?;
        Ok(Self {
            agent,
            fallback_agent,
        })
    }
}

pub struct Data {
    pub http: Client,
    pub ai: Option<AiAgent>,
    pub config: Config,
    pub db: Option<Database>,
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
        let http = Client::new();

        Ok(Self {
            http,
            config,
            ai: None,
            db: None,
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

        Ok(Data {
            http: self.http.unwrap_or_default(),
            config,
            ai: None,
            db: None,
        })
    }
}
