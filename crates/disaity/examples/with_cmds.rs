//! Your own command and a handler, next to the built-in ones.
//!
//! ```sh
//! cp .env.example .env                        # then fill all needed tokens
//! cargo run --example with_cmds
//! ```
//!
//! Commands are plain poise commands. `crate = "disaity::poise"` points the
//! macro at the poise disaity re-exports, so you don't add poise yourself.

use std::sync::atomic::{AtomicU64, Ordering};

use disaity::{WelcomeHandler, commands::Check, prelude::*, tokio};

static PINGS: AtomicU64 = AtomicU64::new(0);

/// replies with how many pings the bot got since it started
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "ping",
    aliases("pg"),
    category = "fun",
    guild_only,
    broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | CONNECT",
    check = "Check::dm_with_auth"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    // use can use call to take any actions
    // let call = ctx.utils().get_or_join_voice().await?;

    let count = PINGS.fetch_add(1, Ordering::Relaxed) + 1;

    say!(ctx, format!("Pong #{count}"));

    drop(typing);

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
        .with_handler(WelcomeHandler)
        .with_data(data)
        .run()
        .await
}
