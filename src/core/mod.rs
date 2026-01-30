pub mod utils;

use ::serenity::all::GatewayIntents;
use poise::{
    Context as BaseContext, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins,
    serenity_prelude as serenity,
};
use songbird::SerenityInit;

use crate::commands::{
    clear::clear, pause::pause, play::play, resume::resume, skip::skip, stop::stop,
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
            commands: vec![play(), pause(), stop(), skip(), clear(), resume()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
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
