# External Binaries

The `bin/` directory is intended for all external **Binaries** required by the bot.

### Setup

Run your bot once with `--install bin` and it will download and unpack all the binaries needed.

- `FFmpeg` GPL build for your platform is fetched from
  [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) and unpacked in `bin/engines/media/`
- `YT-DLP` build for ur platform is fetched from [yt-dlp's own releases](https://github.com/yt-dlp/yt-dlp/releases/latest) and will be placed in `bin/providers/youtube/`

Installs come from each project's own `latest` release and it fetchs for you specific OS.

```bash
cargo run -- --install bin    # while developing; `--` passes the
cargo run -- --update bin     # argument through to your program

./my-bot --install bin        # a shipped binary takes them directly
./my-bot --update bin
```

using `--update bin` will replaces the binares in bin directory.

There is no published build for macOS, so install FFmpeg yourself there and
point `DISAITY_FFMPEG` at it.

### Manual Setup

If you prefer to provide your own binary, place the executable here:

- **Windows:** `ffmpeg.exe` `yt-dlp.exe`
- **Linux/macOS:** `ffmpeg` `yt-dlp`

> [!TIP]
> To use your own builds, put them at `bin/engines/media/ffmpeg` and
> `bin/providers/youtube/yt-dlp`. Just know that `--update bin` will overwrite
> them, since anything in `bin/` counts as disaity's. Point `$DISAITY_YTDLP` /
> `$DISAITY_FFMPEG` at a build you want left alone.

> [!NOTE]
> Note that `--update bin` overwrites it. To keep a build of your own untouched,
> point `DISAITY_FFMPEG` at it instead of placing it here.

### Platform

Support platform are:

- Linux x86-64 / aarch64
- Windows x86-64 / aarch64
