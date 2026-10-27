//! You can use raw command to create your custom command.
//!
//! ```sh
//! cp .env.example .env                        # then fill all needed tokens
//! cargo run --example with_cmds
//! ```

use disaity::{Context, Error, commands::Check, poise::command, prelude::*, tokio};

/// installing poise crate you use the raw_commands method
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "Ping",
    aliases("pg"),
    category = "fun",
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "Check::dm_with_auth"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // let typing = ctx.defer_or_broadcast().await.ok().flatten();

    // use can use call to take any actions
    // let call = ctx.utils().get_or_join_voice().await?;

    // let mut call_lock = call.lock().await;

    say!(ctx, "Pong");

    // drop(typing);

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
        .with_raw_commands([ping()])
        .with_data(data)
        .run()
        .await
}
