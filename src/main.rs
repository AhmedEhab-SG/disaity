use ::serenity::all::GatewayIntents;
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions, builtins, command,
    serenity_prelude as serenity,
};
use songbird::{SerenityInit, input::YoutubeDl};

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

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

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

#[command(slash_command, prefix_command, rename = "stop", aliases("st"))]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = manager.get(guild_id);

    if let Some(handler_lock) = handler_lock {
        let mut handler = handler_lock.lock().await;

        handler.stop();

        ctx.say("⏹ Stopped playback!").await?;

        return Ok(());
    }

    ctx.say("Not in a voice channel!").await?;

    Ok(())
}

#[command(slash_command, prefix_command, rename = "pause", aliases("ps"))]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    let user_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("you must be in a voice channel").await?;
            return Ok(());
        }
    };

    let client_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("I'm not in any channel").await?;
            return Ok(());
        }
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.say("Not in a voice channel!").await?;
            return Ok(());
        }
    };

    let handler = handler_lock.lock().await;

    if let Some(track) = handler.queue().current() {
        track.pause()?;
        ctx.say("⏸️ Paused").await?;
        return Ok(());
    };

    ctx.say("Nothing is playing right now").await?;

    Ok(())
}

#[command(slash_command, prefix_command, rename = "skip", aliases("s"))]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("This command only works in servers.").await?;
            return Ok(());
        }
    };

    // let Some(guild_id) = ctx.guild_id() else {
    //     ctx.say("this commad ont works in servers.").await?;
    //     return Ok(());
    // };

    let user_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("you must be in a voice channel").await?;
            return Ok(());
        }
    };

    let client_ch = match ctx.serenity_context().cache.guild(guild_id).and_then(|g| {
        g.voice_states
            .get(&ctx.serenity_context().cache.current_user().id)
            .and_then(|vs| vs.channel_id)
    }) {
        Some(ch) => ch,
        None => {
            ctx.say("I'm not in any channel").await?;
            return Ok(());
        }
    };

    if client_ch != user_ch {
        ctx.say("You must be in the same voice channel").await?;
        return Ok(());
    }

    let manager = match songbird::get(ctx.serenity_context()).await {
        Some(m) => m,
        None => {
            ctx.say("failed to mount songbird").await?;
            return Ok(());
        }
    };

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.say("Not in a voice channel!").await?;
            return Ok(());
        }
    };

    let handler = handler_lock.lock().await;
    let queue = handler.queue();

    match queue.skip() {
        Ok(_) => ctx.say("skipped").await?,
        Err(_) => ctx.say("failed to skip").await?,
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    #[cfg(any(target_os = "windows", target_family = "unix"))]
    {
        use std::env;

        let current_dir = env::current_dir()?;
        let bin_path = current_dir.join("bin");

        if let Some(path) = env::var_os("PATH") {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();
            paths.insert(0, bin_path);
            let new_path = env::join_paths(paths)?;

            unsafe {
                env::set_var("PATH", &new_path);
            }
        }
    }

    let check = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output();
    match check {
        Ok(_) => println!("✅ FFmpeg is ready to go!"),
        Err(_) => println!("❌ FFmpeg NOT FOUND. Install it or check /bin folder."),
    }

    let check_yt = std::process::Command::new("yt-dlp")
        .arg("-version")
        .output();
    match check_yt {
        Ok(_) => println!("✅ yt-dlp is ready to go!"),
        Err(_) => println!("❌ yt-dlp NOT FOUND. Install it or check /bin folder."),
    }

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
            commands: vec![play(), pause(), stop(), skip()],
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
