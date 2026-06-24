pub mod bin;
pub mod context;
pub mod errors;
pub mod macros;
pub mod utils;

use ::serenity::all::{ClientBuilder, GatewayIntents};
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};

use songbird::SerenityInit;

use crate::{
    commands::CommandsRegistry,
    core::{
        context::Data,
        errors::{Error, on_error_handler},
    },
    handlers::{prayer::start_prayer_loop, ready::start_status_loop},
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

    let data = Data::new();
    let commands = CommandsRegistry::new(&data.config.commands_registry).commands;
    let token = data.config.env.client_token.clone();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(data.config.info_registry.prefix.clone()),
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
                start_status_loop(framework.shard_manager().clone());
                start_prayer_loop(
                    ctx.clone(),
                    data.subscription.prayer_subscription.clone(),
                    data.http.clone(),
                );
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
