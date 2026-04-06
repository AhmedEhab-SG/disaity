pub mod bin;
pub mod context;
pub mod error;
pub mod utils;

use ::serenity::all::{ClientBuilder, GatewayIntents};
use gemini_rust::Gemini;
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};

use songbird::SerenityInit;

use crate::{
    commands::CommandsRegistry,
    config::Config,
    core::{context::Data, error::Error},
    handlers::ready::start_status_loop,
};

pub async fn core() -> Result<(), Error> {
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let config = Config::new();
    let agent = Gemini::new(&config.env.gemini_api_key)?;
    let http = reqwest::Client::new();
    let commands = CommandsRegistry::new(&config.commands_registry).commands;
    let token = config.env.client_token.clone();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands,
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.info_registry.prefix.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                start_status_loop(ctx.clone());
                Ok(Data {
                    http,
                    agent,
                    config,
                })
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
