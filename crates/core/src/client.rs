use std::sync::Arc;

use poise::{Command, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::{
    all::{ClientBuilder, Context as SerenityContext, prelude::GatewayIntents},
    async_trait,
    gateway::ShardManager,
};
use songbird::SerenityInit;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

use super::{
    AsSubscription, Data, Database, Error, ErrorExt, Feature, SubscriptionModule,
    context::{AiAgent, DataBuilder},
};
use disaity_config::Env;

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
    intents: GatewayIntents,
    commands: Vec<Command<Data, Error>>,
    handlers: Vec<Arc<dyn Handler>>,
    features: Vec<Box<dyn Feature>>,
    data: Option<Data>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            intents: Self::default_intents(),
            features: Vec::default(),
            commands: Vec::default(),
            handlers: Vec::default(),
            data: None,
        }
    }
}

impl Client {
    pub fn new() -> Self {
        Self::default()
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

    pub fn with_raw_commands(
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

    pub fn with_tracing(self) -> Self {
        let filter = match (EnvFilter::try_from_default_env(), Env::get_log_level()) {
            (Ok(rust_log), _) => rust_log,
            (Err(_), Some(level)) => EnvFilter::new(format!(
                "warn,\
                 disaity_core={level},\
                 disaity_commands={level},\
                 disaity_config={level},\
                 disaity_handlers={level},\
                 disaity_subscriptions={level},\
                 serenity={level},\
                 poise={level},\
                 songbird={level}"
            )),
            (Err(_), None) => return self,
        };

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .try_init()
            .ok();

        self
    }

    pub async fn run(self) -> Result<(), Error> {
        let Client {
            intents,
            mut commands,
            mut handlers,
            features,
            data,
        } = self;

        let mut data = match data {
            Some(d) => d,
            None => DataBuilder::new().build().await?,
        };

        let span = tracing::error_span!("", message = %data.config.persona.name);
        let setup_span = span.clone();

        async move {
            let token = data
                .config
                .env
                .client_token
                .to_owned()
                .ok_or("missing client token as an env")?;

            let prefix = Some(data.config.info.prefix.to_owned());

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

                data.db = Some(
                    Database::connect(db_path, &data.config.persona.name.to_lowercase()).await?,
                );
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
                            ErrorExt::on_error(error).await.ok();
                        })
                    },
                    ..Default::default()
                })
                .setup(move |ctx, _ready, framework| {
                    Box::pin(
                        async move {
                            builtins::register_globally(ctx, &framework.options().commands).await?;

                            let cx = HandlerCx {
                                serenity: ctx,
                                shared_manager: framework.shard_manager().clone(),
                                data: &data,
                            };

                            tracing::debug!(
                                handlers = handlers.len(),
                                commands = framework.options().commands.len(),
                                "registered, running handler setup"
                            );

                            for handler in &handlers {
                                handler.setup(&cx).await?;
                            }

                            Ok(data)
                        }
                        .instrument(setup_span),
                    )
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
        .instrument(span)
        .await
    }
}
