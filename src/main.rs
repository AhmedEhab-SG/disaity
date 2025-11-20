use ::serenity::all::GatewayIntents;
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions, builtins, command,
    serenity_prelude as serenity,
};
use tokio::main;

struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[command(slash_command, prefix_command, rename = "play", aliases("p"))]
async fn age(ctx: Context<'_>, #[rest] query: String) -> Result<(), Error> {
    ctx.say(format!("Searching for {query}")).await?;
    Ok(())
}

#[main]
async fn main() {
    let token = dotenv::var("CLIENT_TOKEN").unwrap();

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
            commands: vec![age()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
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
        .await;

    client.unwrap().start().await.unwrap();
}
