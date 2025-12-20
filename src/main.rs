use ::serenity::all::GatewayIntents;
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions, builtins, command,
    serenity_prelude as serenity,
};
use songbird::{SerenityInit, input::YoutubeDl};
use tokio::main;

struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[command(slash_command, prefix_command, rename = "play", aliases("p"))]
pub async fn play(ctx: Context<'_>, #[rest] query: String) -> Result<(), Error> {
    let do_search = !query.starts_with("http");

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let voice_channel = ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    });

    let Some(channel_id) = voice_channel else {
        ctx.say("You must be in a voice channel!").await?;
        return Ok(());
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client missing")
        .clone();

    let handler_lock = manager.join(guild_id, channel_id).await?;
    let mut handler = handler_lock.lock().await;

    // create a reqwest client (share it if you're making many requests)
    let client = reqwest::Client::new();

    // Build YoutubeDl source (note: takes Client + url)
    let source = if do_search {
        YoutubeDl::new_search(client, query).into()
    } else {
        YoutubeDl::new(client, query).into()
    };

    // enqueue (songbird 0.5 uses `enqueue`)
    handler.enqueue(source).await;

    ctx.say("▶️ Added to queue!").await?;
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
            commands: vec![play()],
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

    client.unwrap().start().await.unwrap();
}
