# Media Engines Directory

This directory is intended for the **FFmpeg** binaries required by the bot.

### 📥 Automatic Setup

The project's `build.rs` script is configured to automatically download the correct version of FFmpeg for your operating system from the [GitHub Releases](https://github.com/AhmedEhab-SG/disaity/releases) page if it is missing.

### 🛠️ Manual Setup

If you prefer to provide your own binary, place the executable here:

- **Windows:** `ffmpeg.exe`
- **Linux/macOS:** `ffmpeg`

> [!NOTE]
> Files in this directory are ignored by Git to keep the repository size small.
