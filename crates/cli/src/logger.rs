use std::io::{self, IsTerminal};

use tracing_indicatif::{
    IndicatifLayer,
    filter::{IndicatifFilter, hide_indicatif_span_fields},
};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::DefaultFields},
    prelude::*,
};

pub(crate) struct Logger;

impl Logger {
    /// The subscriber for a command: install lines and warnings,
    /// progress bar for each span marked `indicatif.pb_show`.
    /// `RUST_LOG` replaces the filter.
    pub(crate) fn init() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn,disaity_cli=info"));

        let bars = IndicatifLayer::new();

        let lines = fmt::layer()
            .without_time()
            .with_ansi(io::stderr().is_terminal())
            .fmt_fields(hide_indicatif_span_fields(DefaultFields::new()))
            .with_writer(bars.get_stderr_writer())
            .with_filter(filter);

        tracing_subscriber::registry()
            .with(lines)
            .with(bars.with_filter(IndicatifFilter::new(false)))
            .try_init()
            .ok();
    }
}
