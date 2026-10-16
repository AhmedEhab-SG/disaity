//! You can use raw command to create your custom command.
//!
//! ```sh
//! cp .env.example .env                        # then fill all needed tokens
//! cargo run --example with_cmds
//! ```

use disaity::{
    Context, Error,
    commands::checks::{diff_vc, not_mute, user_not_deafen},
    poise::command,
    prelude::*,
    tokio,
};

/// installing poise crate you use the raw_commands method
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "hello_world",
    aliases("hello"),
    category = "fun",
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "diff_vc",
    check = "not_mute",
    check = "user_not_deafen"
)]
pub async fn hello_world(ctx: Context<'_>) -> Result<(), Error> {
    // let _typing = ctx.defer_or_broadcast().await.ok().flatten();

    // use can use call to take any actions
    // let call = ctx.utils().get_or_join_voice().await?;

    // let mut call_lock = call.lock().await;

    ctx.say(
        ctx.data()
            .config
            .persona
            .interactions
            .events
            .guild
            .on_welcome
            .get_random_res()
            .unwrap_or("Hello Word"),
    )
    .await?;

    Ok(())
}

#[tokio::main(crate = "disaity::tokio")]
async fn main() -> Result<(), Error> {
    Binaries::ensure().await?;

    let data = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_info(Info::new().with_prefix("~"))
                .with_persona(Persona::new(Preset::Emilia))
                .build(),
        )
        .build()
        .await?;

    Client::new()
        .with_tracing()
        .with_feature(Commands::all())
        // .with_raw_commands([hello_world()])
        .with_data(data)
        .run()
        .await
}
