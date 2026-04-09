pub mod bin;
pub mod context;
pub mod error;
pub mod utils;

use ::serenity::all::{ClientBuilder, GatewayIntents};
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};

use songbird::SerenityInit;

use crate::{
    commands::CommandsRegistry,
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

    let data = Data::new();
    let commands = CommandsRegistry::new(&data.config.commands_registry).commands;
    let token = data.config.env.client_token.clone();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            // handles only check errors for now
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        // This catches errors returned specifically from 'check' functions
                        poise::FrameworkError::CommandCheckFailed { ctx, error, .. } => {
                            if let Some(err_msg) = error {
                                ctx.say(format!("{}", err_msg)).await.ok();
                            }
                        }
                        // Handle other errors...
                        _ => poise::builtins::on_error(error).await.unwrap(),
                    }
                })
            },
            commands,
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(data.config.info_registry.prefix.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                start_status_loop(ctx.clone());
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
