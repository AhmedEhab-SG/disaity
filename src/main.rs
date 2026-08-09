use serenity::all::{ClientBuilder, GatewayIntents};
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use songbird::SerenityInit;

use disaity::{
    commands::CommandsRegistry,
    core::{BinariesExt, Data, Error, on_error_handler},
    handlers::{start_prayer_loop, start_status_loop},
};

async fn create() -> Result<(), Error> {
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let data = Data::new();

    let token = data.config.env.client_token.clone();
    let commands = CommandsRegistry::new(&data.config.commands_registry).commands;
    let subscription = data.subscription.clone();

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
                start_prayer_loop(ctx.clone(), subscription, data.http.clone());
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    BinariesExt::load();

    create().await.unwrap();
}
