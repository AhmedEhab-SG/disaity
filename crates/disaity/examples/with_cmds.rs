//! Your own command and a handler, next to the built-in ones.
//!
//! ```sh
//! cp .env.example .env                        # then fill CLIENT_TOKEN
//! cargo run --example with_cmds
//! ```
//!
//! Commands are poise commands. `crate = "disaity::poise"` to use the rexport

use std::sync::atomic::{AtomicI64, Ordering};

use disaity::{prelude::*, tokio};

// counter
static COUNT: AtomicI64 = AtomicI64::new(0);

// placeholder main cmd for subcmds
/// a counter everyone shares
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "count",
    aliases("ct"),
    category = "counter",
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommand_required,
    subcommands("counter_plus", "counter_minus", "counter_set", "counter_clear")
)]
pub async fn counter(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// add one to the counter
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "plus"
)]
pub async fn counter_plus(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    drop(typing);

    say!(ctx, format!("Counter #{count}"));

    Ok(())
}

/// take one from the counter
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "minus"
)]
pub async fn counter_minus(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    let count = COUNT.fetch_sub(1, Ordering::Relaxed) - 1;

    drop(typing);

    say!(ctx, format!("Counter #{count}"));

    Ok(())
}

/// set the counter to a number
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "set"
)]
pub async fn counter_set(
    ctx: Context<'_>,
    #[description = "The number to set the counter to"] count: i64,
) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    COUNT.store(count, Ordering::Relaxed);

    drop(typing);

    say!(ctx, format!("Counter #{count}"));

    Ok(())
}

/// reset the counter to zero
#[command(
    crate = "disaity::poise",
    slash_command,
    prefix_command,
    rename = "clear"
)]
pub async fn counter_clear(ctx: Context<'_>) -> Result<(), Error> {
    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    COUNT.store(0, Ordering::Relaxed);

    drop(typing);

    say!(ctx, "Counter #0");

    Ok(())
}

#[tokio::main(crate = "disaity::tokio")]
async fn main() -> Result<(), Error> {
    // default prefix is `!`
    Client::new()
        .with_tracing()
        .with_raw_commands([counter()]) // add your commands with there subcommands
        .run()
        .await
}
