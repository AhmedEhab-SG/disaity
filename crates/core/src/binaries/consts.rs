use super::spec::{BinarySpec, Download};

pub(super) const BIN_DIR: &str = "bin";

/// Names a single directory that holds `bin/`, skipping the usual search.
pub(super) const HOME_VAR: &str = "DISAITY_HOME";

pub const FFMPEG: BinarySpec = BinarySpec {
    name: "ffmpeg",
    segments: &["engines", "media"],
    override_var: "DISAITY_FFMPEG",
    base_url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest",
    downloads: &[
        Download::new("linux", "x86_64", "ffmpeg-master-latest-linux64-gpl.tar.xz"),
        Download::new(
            "linux",
            "aarch64",
            "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
        ),
        Download::new("windows", "x86_64", "ffmpeg-master-latest-win64-gpl.zip"),
        Download::new(
            "windows",
            "aarch64",
            "ffmpeg-master-latest-winarm64-gpl.zip",
        ),
    ],
};

pub const YTDLP: BinarySpec = BinarySpec {
    name: "yt-dlp",
    segments: &["providers", "youtube"],
    override_var: "DISAITY_YTDLP",
    base_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download",
    downloads: &[
        Download::new("linux", "x86_64", "yt-dlp_linux"),
        Download::new("linux", "aarch64", "yt-dlp_linux_aarch64"),
        Download::new("linux", "arm", "yt-dlp_linux_armv7l"),
        Download::new("macos", "x86_64", "yt-dlp_macos"),
        Download::new("macos", "aarch64", "yt-dlp_macos"),
        Download::new("windows", "x86_64", "yt-dlp.exe"),
        Download::new("windows", "x86", "yt-dlp_x86.exe"),
    ],
};

/// Every binary the framework resolves, in the order it reports them.
pub(super) const ALL_BIN: &[&BinarySpec] = &[&FFMPEG, &YTDLP];
