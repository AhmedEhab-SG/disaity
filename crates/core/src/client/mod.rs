mod event;
mod handler;
mod logger;

use std::sync::Arc;

use poise::{Command, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::all::{ClientBuilder, prelude::GatewayIntents};
use songbird::SerenityInit;
use tracing::Instrument;

use disaity_cli::{Binaries, Interface};

use crate::client::logger::Traced;

use super::{
    AiAgent, AiUrl, AsSubscription, Data, Database, Error, ErrorExt, Feature, SubscriptionModule,
    context::DataBuilder,
};

use event::{Handlers, dispatch};
pub use handler::{Handler, HandlerCx};
use logger::Logger;

pub struct Client {
    intents: GatewayIntents,
    commands: Vec<Command<Data, Error>>,
    handlers: Vec<Arc<dyn Handler>>,
    features: Vec<Box<dyn Feature>>,
    data: Option<Data>,
    traced: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            intents: Self::default_intents(),
            features: Vec::default(),
            commands: Vec::default(),
            handlers: Vec::default(),
            data: None,
            traced: false,
        }
    }
}

impl Client {
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

    pub fn new() -> Self {
        Self::default()
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

    pub fn with_tracing(mut self) -> Self {
        self.traced = true;
        self
    }

    pub async fn run(self) -> Result<(), Error> {
        // reqwest 0.12 turns on rustls' ring and genai's reqwest 0.13 turns on aws-lc-rs.
        // with both, rustls can't pick one and songbird's voice socket panics, so pick ring
        // TODO: remove with the rustls dep after the serenity next release
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();

        if Interface::run_once().await? {
            return Ok(());
        }

        let Client {
            intents,
            mut commands,
            mut handlers,
            features,
            data,
            traced,
        } = self;

        if traced {
            Logger::init();
        }

        let mut data = match data {
            Some(d) => d,
            None => DataBuilder::new().build().await?,
        };

        let name = &data.config.persona.name;

        // the empty traced span to read it on loggger
        let span = if traced {
            tracing::error_span!("", message = %name, traced = tracing::field::Empty)
        } else {
            tracing::error_span!("", message = %name)
        };

        async move {
            let token = data
                .config
                .env
                .client_token
                .to_owned()
                .ok_or("missing client token as an env")?;

            let prefix = Some(data.config.info.prefix.to_owned());

            // ensure bin on demand
            if features.iter().any(|feature| feature.needs_bin()) {
                Binaries::ensure().await?;
            }

            // connect ai on demand
            if features.iter().any(|feature| feature.needs_ai()) {
                let url = data
                    .config
                    .env
                    .ai_url
                    .as_deref()
                    .ok_or("an AI feature is registered but AI_URL is not set")?
                    .parse::<AiUrl>()?;

                data.ai = Some(AiAgent::connect(&url)?);
            }

            // connect db on demand
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

            let event_handlers = handlers.clone();

            let framework = Framework::builder()
                .options(FrameworkOptions {
                    prefix_options: PrefixFrameworkOptions {
                        prefix: prefix.map(Into::into),
                        ..Default::default()
                    },

                    commands,

                    event_handler: |framework, event| Box::pin(dispatch(framework, event)),

                    on_error: |error| {
                        Box::pin(async move {
                            ErrorExt::on_error(error).await.ok();
                        })
                    },
                    ..Default::default()
                })
                .setup(move |ctx, _ready, framework| {
                    Box::pin(async move {
                        builtins::register_globally(ctx, &framework.options().commands).await?;

                        let cx = HandlerCx {
                            serenity: ctx,
                            shard_manager: framework.shard_manager().clone(),
                            data: &data,
                        };

                        tracing::debug!(
                            handlers = handlers.len(),
                            commands = framework.options().commands.len(),
                            features = features.len(),
                            traced = traced,
                            "registered, running handler setup"
                        );

                        for handler in &handlers {
                            handler.setup(&cx).await?;
                        }

                        Ok(data)
                    })
                })
                .build();

            ClientBuilder::new(token, intents)
                .framework(Traced::new(framework))
                .type_map_insert::<Handlers>(event_handlers)
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
