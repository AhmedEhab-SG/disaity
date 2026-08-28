mod archive;
mod args;
mod consts;
mod fetch;
mod install;
mod resolve;
mod spec;
mod utils;

use std::{process, sync::OnceLock};

use crate::errors::Error;

use args::Action;

pub use consts::{FFMPEG, YTDLP};
pub use resolve::ResolvedBinary;
pub use spec::BinarySpec;

pub struct Binaries {
    pub ffmpeg: ResolvedBinary,
    pub ytdlp: ResolvedBinary,
}

static BINARIES: OnceLock<Binaries> = OnceLock::new();

impl Binaries {
    /// Resolve every declared binary and cache the result process-wide. Call
    /// this once at startup, before any client runs; repeat calls are cheap.
    ///
    /// A program that cannot be found is an error naming the command to run.
    /// `--install bin` and `--update bin` are handled here and exit the
    /// process, since they are the reason it started.
    pub async fn ensure() -> Result<&'static Self, Error> {
        if let Some(action) = Action::parse() {
            install::run(action).await?;
            process::exit(0);
        }

        if let Some(binaries) = BINARIES.get() {
            return Ok(binaries);
        }

        let binaries = Self {
            ffmpeg: ResolvedBinary::locate(&FFMPEG)?,
            ytdlp: ResolvedBinary::locate(&YTDLP)?,
        };

        for resolved in [&binaries.ffmpeg, &binaries.ytdlp] {
            println!("{resolved}");
        }

        Ok(BINARIES.get_or_init(|| binaries))
    }

    /// The set resolved by [`Binaries::ensure`].
    pub fn get() -> Result<&'static Self, Error> {
        BINARIES.get().ok_or_else(|| {
            "binaries are not ready: call `Binaries::ensure()` during startup".into()
        })
    }
}
