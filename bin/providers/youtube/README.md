# YouTube Provider Binaries

This directory is intended for the **yt-dlp** binaries used to fetch and stream YouTube content.

### 📥 Automatic Setup

If this folder is empty, the `build.rs` script will automatically download the correct version of **yt-dlp** for your system during the next `cargo build` or `cargo run`.

Downloads are fetched from the official project [GitHub Releases](https://github.com/AhmedEhab-SG/disaity/releases).

### 🛠️ Manual Setup

To use a specific version, place the executable here:

- **Windows:** `yt-dlp.exe`
- **Linux/macOS:** `yt-dlp`

> [!NOTE]
> This folder contains a `.gitignore` to ensure these large binaries are not uploaded to the repository.
