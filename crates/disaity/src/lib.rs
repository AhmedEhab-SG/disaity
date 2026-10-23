//! Disaity — a batteries-included Discord bot framework for music and AI chat,
//! built on [`serenity`], [`poise`] and [`songbird`].
//!
//! ```no_run
//! use disaity::prelude::*;
//!
//! # async fn run() -> Result<(), Error> {
//! Binaries::ensure().await?;
//!
//! let data = DataBuilder::new()
//!     .with_config(
//!         ConfigBuilder::new()
//!             .with_info(Info::new().with_prefix("!"))
//!             .with_persona(Persona::new(Preset::Emilia))
//!             .build(),
//!     )
//!     .build()
//!     .await?;
//!
//! Client::new()
//!     .with_feature(Commands::music())
//!     .with_data(data)
//!     .run()
//!     .await
//! # }
//! ```
//!
//! # External binaries
//!
//! Run your bot once with `--install bin` to fetch the current upstream build
//! of each into, and `--update bin` to replace them later if music needed.

pub mod prelude;
pub use disaity_commands as commands;
pub use disaity_config as config;
pub use disaity_core as core;
pub use disaity_handlers as handlers;
pub use disaity_subscriptions as subscriptions;

pub use disaity_core::{poise, reqwest, serde, serenity, songbird, sqlx, tokio};

pub use disaity_commands::Commands;
pub use disaity_config::{
    ActivityType, AiAccess, Assets, Category, CommandId, CommandRegistry, Config, ConfigBuilder,
    Env, Info, LogLevel, Persona, Platform, Preset, Provider, Status,
};
pub use disaity_core::{
    AsSubscription, Binaries, BinarySpec, Client, Context, ContextExt, Data, DataBuilder, Database,
    Decorate, Error, ErrorExt, FFMPEG, Feature, FeatureBuilder, Handler, HandlerCx, ReactionUtils,
    ResolvedBinary, SubscriptionModule, TrackMetadata, VoiceUtils, YTDLP, say, send,
};
pub use disaity_handlers::StatusHandler;
pub use disaity_subscriptions::FreeGamesSubscription;
