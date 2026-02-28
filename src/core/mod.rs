pub mod utils;

use ::serenity::all::GatewayIntents;
use poise::{
    Context as BaseContext, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins,
    serenity_prelude as serenity,
};
use songbird::SerenityInit;

use crate::{
    commands::{
        music::{
            clear::clear, jump::jump, pause::pause, play::play, queue::queue, repeat::repeat,
            resume::resume, seek::seek, shuffle::shuffle, skip::skip, stop::stop,
        },
        others::{help::help, join::join, leave::leave},
    },
    handlers::ready::start_status_loop,
};

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = BaseContext<'a, Data, Error>;

pub async fn core() -> Result<(), Error> {
    let token = dotenv::var("CLIENT_TOKEN")?;

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                play(),
                pause(),
                stop(),
                skip(),
                clear(),
                resume(),
                help(),
                join(),
                leave(),
                jump(),
                repeat(),
                queue(),
                shuffle(),
                seek(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                start_status_loop(ctx.clone());
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await;

    client?.start().await?;

    Ok(())
}
