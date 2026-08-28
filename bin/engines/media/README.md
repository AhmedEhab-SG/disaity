# Media Engines Directory

This directory is intended for the **FFmpeg** binary required by the bot.

### 📥 Setup

Nothing is downloaded on its own. Run your bot once with `--install bin` and the
full GPL build for your platform is fetched from
[BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) and unpacked here:

```bash
cargo run -- --install bin    # fetch it if this directory is empty
cargo run -- --update bin     # replace it with the current upstream build
```

`--update bin` only replaces a copy from this directory, and never installs one.
A system FFmpeg is reported and left alone — update that one with your package
manager. With this directory empty, it tells you to run `--install bin` first.

There is no published build for macOS, so install FFmpeg yourself there and
point `DISAITY_FFMPEG` at it.

### 🛠️ Manual Setup

If you prefer to provide your own binary, place the executable here:

- **Windows:** `ffmpeg.exe`
- **Linux/macOS:** `ffmpeg`

Note that `--update bin` overwrites it. To keep a build of your own untouched,
point `DISAITY_FFMPEG` at it instead of placing it here.

> [!NOTE]
> Files in this directory are ignored by Git to keep the repository size small.
