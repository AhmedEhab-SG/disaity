# YouTube Provider Binaries

This directory is intended for the **yt-dlp** binary used to fetch and stream
YouTube content.

### 📥 Setup

Nothing is downloaded on its own. Run your bot once with `--install bin` and the
standalone build for your platform is fetched from
[yt-dlp's own releases](https://github.com/yt-dlp/yt-dlp/releases/latest):

```bash
cargo run -- --install bin    # fetch it if this directory is empty
cargo run -- --update bin     # replace it with the current upstream build
```

Reach for `--update bin` when playback starts failing with `403` — that is
almost always a yt-dlp that YouTube has moved on from.

`--update bin` only replaces a copy from this directory, and never installs one.
If the copy in use is a system install it says so and leaves it alone; update
that one with your package manager, or run `--install bin` to have disaity keep
its own here instead — that one wins over `PATH`, and you can update it freely.

### 🛠️ Manual Setup

To use a specific version, place the executable here:

- **Windows:** `yt-dlp.exe`
- **Linux/macOS:** `yt-dlp`

Note that `--update bin` overwrites it. To keep a build of your own untouched,
point `DISAITY_YTDLP` at it instead of placing it here.

> [!NOTE]
> These large binaries are ignored by Git so they are never uploaded to the
> repository.
