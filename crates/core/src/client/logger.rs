use std::{str::FromStr, sync::Once};

use serenity::all::{
    Client as SerenityClient, Context as SerenityContext, Framework, FullEvent, async_trait,
};
use tracing::{Instrument, Span, level_filters::LevelFilter};
use tracing_subscriber::{
    EnvFilter,
    filter::{FilterExt, Targets},
    fmt,
    prelude::*,
};

use disaity_config::{Env, LogLevel};

const TARGETS: &[&str] = &[
    "disaity_core",
    "disaity_commands",
    "disaity_config",
    "disaity_handlers",
    "disaity_subscriptions",
    "serenity",
    "poise",
    "songbird",
];

static INIT: Once = Once::new();

pub(super) struct Logger;

impl Logger {
    /// client with `with_tracing()`: `LOG_LEVEL` for the crates above
    /// `RUST_LOG` replaces all of it, for when you'd rather write the filter.
    pub(super) fn init() {
        INIT.call_once(|| {
            let layer = fmt::layer().with_target(false);

            let installed = match EnvFilter::try_from_default_env() {
                Ok(manual) => tracing_subscriber::registry()
                    .with(layer.with_filter(manual))
                    .try_init(),

                Err(_) => {
                    // unset means OFF
                    let level = Env::get_log_level().map_or(LevelFilter::OFF, Self::level);

                    let base = EnvFilter::new(format!(
                        "{},disaity_cli=info",
                        level.min(LevelFilter::WARN)
                    ));

                    let traced = EnvFilter::new(format!("[{{traced}}]={level}"));

                    let ours =
                        Targets::new().with_targets(TARGETS.iter().map(|target| (*target, level)));

                    tracing_subscriber::registry()
                        .with(layer.with_filter(base.or(traced.and(ours))))
                        .try_init()
                }
            };

            installed.ok();
        });
    }

    fn level(level: LogLevel) -> LevelFilter {
        LevelFilter::from_str(level.as_ref())
            .inspect_err(|e| tracing::error!(%e, "invalid log level, defaulting to `DEBUG`"))
            .unwrap_or(LevelFilter::DEBUG)
    }
}

/// wapper around, poise and every dispatch attach span on it with persona
pub(super) struct Traced<F> {
    inner: F,
    span: Span,
}

impl<F> Traced<F> {
    /// Takes the span it's created in, `Client::run`'s.
    pub(super) fn new(inner: F) -> Self {
        Self {
            inner,
            span: Span::current(),
        }
    }
}

#[async_trait]
impl<F: Framework> Framework for Traced<F> {
    async fn init(&mut self, client: &SerenityClient) {
        let span = self.span.clone();
        self.inner.init(client).instrument(span).await;
    }

    async fn dispatch(&self, ctx: SerenityContext, event: FullEvent) {
        self.inner
            .dispatch(ctx, event)
            .instrument(self.span.clone())
            .await;
    }
}
