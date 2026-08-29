//! Disaity — a batteries-included wrapper over [`serenity`], [`poise`] and
//! [`songbird`] for building Discord music and chat bots.
//!
//! ```no_run
//! use disaity::prelude::*;
//!
//! # async fn run() -> Result<(), Error> {
//! Binaries::ensure().await?;
//!
//! let data = DataBuilder::new()
//!     .with_config(ConfigBuilder::new().with_persona(Persona::new(Preset::Emilia)).build())
//!     .build()
//!     .await?;
//!
//! Client::new(&data.config.env.client_token)
//!     .with_prefix(&data.config.info.prefix)
//!     .with_feature(Commands::music())
//!     .with_data(data)
//!     .run()
//!     .await
//! # }
//! ```
//!
//! # External binaries
//!
//! Playback shells out to `yt-dlp` and `ffmpeg`. [`Binaries::ensure`] locates
//! them, in this order:
//!
//! 1. `DISAITY_FFMPEG` / `DISAITY_YTDLP`, if you want to point at your own copy
//! 2. `$DISAITY_HOME/bin/…`
//! 3. `bin/…` next to your crate's `Cargo.toml` — how `cargo run` resolves
//! 4. `bin/…` next to the executable — how a shipped build resolves
//! 5. `bin/…` under your platform's per-user data directory
//! 6. whatever is on the system `PATH`
//!
//! Nothing is ever downloaded implicitly — not at build time, not at startup.
//! Run your bot once with `--install bin` to fetch the current upstream build
//! of each into (3), and `--update bin` to replace them later. Both arguments
//! are handled by [`Binaries::ensure`] before anything connects to Discord.
//!
//! `--install bin` always takes a copy of its own, even where the system has
//! one, so playback does not depend on how current your distro is. `--update
//! bin` only refreshes that copy and never installs: a system build or one
//! behind (1) is reported and left alone, and with nothing installed at all it
//! points you back at `--install bin`.

pub mod prelude;
pub use disaity_commands as commands;
pub use disaity_config as config;
pub use disaity_core as core;
pub use disaity_handlers as handlers;
pub use disaity_subscriptions as subscriptions;

pub use disaity_core::{poise, serenity, songbird};

pub use disaity_commands::Commands;
pub use disaity_config::{
    ActivityType, Assets, Category, CommandId, CommandRegistry, Config, ConfigBuilder, Env, Info,
    LogLevel, Persona, Preset, Provider, Status,
};
pub use disaity_core::{
    AsSubscription, Binaries, BinarySpec, Client, Context, ContextExt, Data, DataBuilder, Database,
    Decorate, Error, FFMPEG, Feature, FeatureBuilder, Handler, HandlerCx, ReactionUtils,
    ResolvedBinary, SongMetadata, SubscriptionModule, VoiceUtils, YTDLP, on_error_handler, say,
};
pub use disaity_handlers::StatusHandler;
pub use disaity_subscriptions::PrayerModule;
