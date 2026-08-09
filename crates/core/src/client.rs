use std::sync::Arc;

use poise::{Command, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins, framework};
use serenity::{
    all::{ClientBuilder, Context as SerenityContext, prelude::GatewayIntents},
    async_trait,
    gateway::ShardManager,
};
use songbird::SerenityInit;

use crate::{Data, Error, context::DataBuilder, on_error_handler};

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
    data: Option<Data>,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            intents: Self::default_intents(),
            prefix: None,
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
            commands,
            handlers,
            data,
        } = self;

        let data = data.unwrap_or_else(|| DataBuilder::new().build());

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
