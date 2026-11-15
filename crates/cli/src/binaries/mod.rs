mod archive;
mod consts;
mod fetch;
mod install;
mod resolve;
mod spec;
mod utils;

use anyhow::{Context, Result};
use tokio::sync::OnceCell;

use disaity_config::Bin;

pub use consts::{FFMPEG, YTDLP};
pub(crate) use install::{Install, Mode};
pub use resolve::ResolvedBinary;
pub use spec::BinarySpec;
pub(crate) use utils::Caller;

pub struct Binaries;

static FFMPEG_FOUND: OnceCell<ResolvedBinary> = OnceCell::const_new();
static YTDLP_FOUND: OnceCell<ResolvedBinary> = OnceCell::const_new();

impl Binaries {
    /// Find every binary the bot runs, once per process. Two clients starting
    /// together share one search. A failed search isn't kept, so the next call
    /// searches again.
    pub async fn ensure(bins: &[Bin]) -> Result<()> {
        let home = Caller::Client.home(None)?;

        for &bin in bins {
            Self::found(bin)
                .get_or_try_init(|| async {
                    let resolved_bin = ResolvedBinary::locate(Self::spec(bin), &home)?;

                    tracing::info!("{resolved_bin}");

                    Ok::<_, anyhow::Error>(resolved_bin)
                })
                .await?;
        }

        Ok(())
    }

    /// The set resolved by [`Binaries::ensure`].
    pub fn ytdlp() -> Result<&'static ResolvedBinary> {
        YTDLP_FOUND
            .get()
            .context("yt-dlp isn't ready: no registered command lists it in `needs.bin`")
    }

    /// Extra args every spawned yt-dlp gets.
    pub fn ytdlp_args() -> Vec<String> {
        let mut args = vec!["--js-runtimes".to_string(), "node".to_string()];

        if let Some(dir) = YTDLP_FOUND
            .get()
            .and_then(ResolvedBinary::dir)
            .filter(|dir| !dir.as_os_str().is_empty())
        {
            args.extend(["--ffmpeg-location".to_string(), dir.display().to_string()]);
        }
        args
    }

    //INFO: could also expose ffmpg if needed with custom args

    pub(crate) fn spec(bin: Bin) -> &'static BinarySpec {
        match bin {
            Bin::Ffmpeg => &FFMPEG,
            Bin::YtDlp => &YTDLP,
        }
    }

    fn found(bin: Bin) -> &'static OnceCell<ResolvedBinary> {
        match bin {
            Bin::Ffmpeg => &FFMPEG_FOUND,
            Bin::YtDlp => &YTDLP_FOUND,
        }
    }
}
