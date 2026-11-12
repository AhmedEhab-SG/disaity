//! Disaity — a batteries-included Discord bot framework for music and AI chat,
//! built on [`serenity`], [`poise`] and [`songbird`].
//!
//! ```no_run
//! use disaity::prelude::*;
//!
//! # async fn run() -> Result<(), Error> {
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
//!     .with_feature(Commands::new().with(CommandId::Ping))
//!     .with_data(data)
//!     .run()
//!     .await
//! # }
//! ```

pub mod prelude;
use disaity_cli as cli;
use disaity_commands as commands;
use disaity_config as config;
use disaity_core as core;
use disaity_handlers as handlers;
use disaity_subscriptions as subscriptions;

pub use cli::{Binaries, BinarySpec, FFMPEG, Interface, ResolvedBinary, YTDLP};
pub use commands::Commands;
pub use config::{
    ActivityType, AiAccess, Assets, Category, CommandId, CommandRegistry, Config, ConfigBuilder,
    Env, Info, LogLevel, Persona, Platform, Preset, Provider, Status,
};
pub use core::{
    AiAgent, AiUrl, AsSubscription, Client, CommandMeta, Context, ContextExt, Data, DataBuilder,
    Database, Decorate, Error, ErrorExt, Feature, FeatureBuilder, Handler, HandlerCx,
    ReactionUtils, SubscriptionModule, TrackMetadata, VoiceUtils, genai, poise, reply, reqwest,
    say, send, serde, serenity, songbird, sqlx, tokio,
};
pub use handlers::{StatusHandler, WelcomeHandler};
pub use subscriptions::FreeGamesSubscription;
