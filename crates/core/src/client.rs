use std::sync::Arc;

use poise::{Command, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::{
    all::{ClientBuilder, Context as SerenityContext, prelude::GatewayIntents},
    async_trait,
    gateway::ShardManager,
};
use songbird::SerenityInit;

use crate::{
    AsSubscription, Data, Database, Error, Feature, SubscriptionModule,
    context::{AiAgent, DataBuilder},
    on_error_handler,
};

pub struct HandlerCx<'a> {
    pub serenity: &'a SerenityContext,
    pub shared_manager: Arc<ShardManager>,
    pub data: &'a Data,
}

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn setup(&self, cx: &HandlerCx<'_>) -> Result<(), Error>;
}

pub struct Client {
    token: String,
    intents: GatewayIntents,
    prefix: Option<String>,
    commands: Vec<Command<Data, Error>>,
    handlers: Vec<Arc<dyn Handler>>,
    features: Vec<Box<dyn Feature>>,
    data: Option<Data>,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            intents: Self::default_intents(),
            prefix: None,
            features: Vec::new(),
            commands: Vec::new(),
            handlers: Vec::new(),
            data: None,
        }
    }

    fn default_intents() -> GatewayIntents {
        GatewayIntents::non_privileged()
            | GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::GUILD_INTEGRATIONS
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::DIRECT_MESSAGES
    }

    pub fn with_intents(mut self, intents: GatewayIntents) -> Self {
        self.intents = intents;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn with_commands(
        mut self,
        commands: impl IntoIterator<Item = Command<Data, Error>>,
    ) -> Self {
        self.commands.extend(commands);
        self
    }

    pub fn with_feature(mut self, feature: impl Feature + 'static) -> Self {
        self.features.push(Box::new(feature));
        self
    }

    pub fn with_subscription(self, sub: impl SubscriptionModule) -> Self {
        self.with_feature(AsSubscription(sub))
    }

    pub fn with_handler(mut self, handler: impl Handler) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }

    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    pub async fn run(self) -> Result<(), Error> {
        let Client {
            token,
            intents,
            prefix,
            mut commands,
            mut handlers,
            features,
            data,
        } = self;

        let mut data = match data {
            Some(d) => d,
            None => DataBuilder::new().build().await?,
        };

        if features.iter().any(|feature| feature.needs_ai()) {
            let key = data
                .config
                .env
                .gemini_api_key
                .as_deref()
                .ok_or("an AI feature is registered but GEMINI_API_KEY is not set")?;

            data.ai = Some(AiAgent::connect(key)?);
        }

        if features.iter().any(|feature| feature.needs_db()) {
            let db_path = data
                .config
                .env
                .db_path
                .as_deref()
                .ok_or("a subscription feature is registered but DB_PATH is not set")?;

            data.db = Some(Database::connect(db_path, &data.config.persona.name).await?);
        }

        for feature in &features {
            if !feature.enabled(&data) {
                continue;
            }

            commands.extend(feature.commands(&data));
            if let Some(handler) = feature.handler() {
                handlers.push(handler);
            }
        }

        let framework = Framework::builder()
            .options(FrameworkOptions {
                prefix_options: PrefixFrameworkOptions {
                    prefix,
                    ..Default::default()
                },
                commands,
                on_error: |error| {
                    Box::pin(async move {
                        on_error_handler(error).await.ok();
                    })
                },
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    builtins::register_globally(ctx, &framework.options().commands).await?;

                    let cx = HandlerCx {
                        serenity: ctx,
                        shared_manager: framework.shard_manager().clone(),
                        data: &data,
                    };

                    for handler in &handlers {
                        handler.setup(&cx).await?;
                    }

                    Ok(data)
                })
            })
            .build();

        ClientBuilder::new(token, intents)
            .framework(framework)
            .register_songbird()
            .await?
            .start()
            .await?;

        Ok(())
    }
}
