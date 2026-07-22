mod bin;
mod context;
mod errors;
pub mod macros;
mod utils;

use ::serenity::all::{ClientBuilder, GatewayIntents};
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};

use songbird::SerenityInit;

use crate::handlers::{prayer::start_prayer_loop, ready::start_status_loop};
pub use bin::BinariesExt;
pub use context::{Context, ContextExt, Data};
pub use errors::{Error, on_error_handler};
pub use utils::{ReactionUtils, VoiceUtils};

pub async fn create() -> Result<(), Error> {
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut data = Data::new();

    let token = data.config.env.client_token.clone();
    let commands = std::mem::take(&mut data.registry.commands);
    let subscription = std::mem::take(&mut data.registry.subscription);

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
