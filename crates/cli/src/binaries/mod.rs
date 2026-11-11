mod archive;
mod consts;
mod fetch;
mod install;
mod resolve;
mod spec;
mod utils;

use anyhow::{Context, Result};
use tokio::sync::OnceCell;

pub(crate) use consts::ALL_BIN;
pub use consts::{FFMPEG, YTDLP};
pub(crate) use install::{Install, Mode};
pub use resolve::ResolvedBinary;
pub use spec::BinarySpec;

pub struct Binaries {
    pub ffmpeg: ResolvedBinary,
    pub ytdlp: ResolvedBinary,
}

static BINARIES: OnceCell<Binaries> = OnceCell::const_new();

impl Binaries {
    /// Find every binary the bot runs, once per process. Two clients starting
    /// together share one search. A failed search isn't kept, so the next call
    /// searches again.
    pub async fn ensure() -> Result<&'static Self> {
        BINARIES
            .get_or_try_init(|| async {
                let binaries = Self {
                    ffmpeg: ResolvedBinary::locate(&FFMPEG)?,
                    ytdlp: ResolvedBinary::locate(&YTDLP)?,
                };

                for resolved_bin in [&binaries.ffmpeg, &binaries.ytdlp] {
                    tracing::info!("{resolved_bin}");
                }

                Ok(binaries)
            })
            .await
    }

    /// The set resolved by [`Binaries::ensure`].
    pub fn get() -> Result<&'static Self> {
        BINARIES
            .get()
            .with_context(|| "binaries are not ready: call `Binaries::ensure()` during startup")
    }

    /// Extra args every spawned yt-dlp gets.
    pub fn ytdlp_args(&self) -> Vec<String> {
        let mut args = vec!["--js-runtimes".to_string(), "node".to_string()];
        if let Some(dir) = self.ffmpeg.dir().filter(|dir| !dir.as_os_str().is_empty()) {
            args.extend(["--ffmpeg-location".to_string(), dir.display().to_string()]);
        }
        args
    }
}
