use std::any::{Any, type_name};

use poise::Context as BaseContext;
use reqwest::Client;
use serenity::async_trait;

use disaity_config::Config;

use super::{AiAgent, Database, Error, Utils};

pub type Context<'ctx> = BaseContext<'ctx, Data, Error>;
pub type State = Box<dyn Any + Send + Sync>;

pub struct Data {
    pub http: Client,
    pub ai: Option<AiAgent>,
    pub config: Config,
    pub db: Option<Database>,
    state: Option<State>,
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
            state: None,
        })
    }

    /// get ref from user data wanted to share
    pub fn state<T>(&self) -> Result<&T, Error>
    where
        T: Send + Sync + 'static,
    {
        self.state
            .as_deref()
            .and_then(|state| state.downcast_ref::<T>())
            .ok_or_else(|| {
                format!(
                    "Data has no `{}` state, add it with DataBuilder::with_state",
                    type_name::<T>()
                )
                .into()
            })
    }
}

#[derive(Default)]
pub struct DataBuilder {
    config: Option<Config>,
    http: Option<Client>,
    state: Option<State>,
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

    /// set a user data wanted to share
    pub fn with_state<T>(mut self, state: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.state = Some(Box::new(state));
        self
    }

    pub async fn build(self) -> Result<Data, Error> {
        let config = self.config.unwrap_or_default();

        Ok(Data {
            http: self.http.unwrap_or_default(),
            config,
            ai: None,
            db: None,
            state: None,
        })
    }
}
