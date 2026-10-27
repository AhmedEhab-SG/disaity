<div align=center>
    <img src="./assets/logo.png" width="200" alt="ferris-logo" />

<h3>Disaity</h3>

<p>
    A Rust framework for building Discord music and chat bots, on top of
    <a href="https://github.com/serenity-rs/serenity" target="_blank">serenity</a>,
    <a href="https://github.com/serenity-rs/poise" target="_blank">poise</a> and
    <a href="https://github.com/serenity-rs/songbird" target="_blank">songbird</a>.
</p>

</div>

<div align="center">

[![CI](https://github.com/AhmedEhab-SG/disaity/actions/workflows/ci.yml/badge.svg)](https://github.com/AhmedEhab-SG/disaity/actions/workflows/ci.yml)
[![Status](https://img.shields.io/badge/status-experimental-7c1f1f?logo=labex&logoColor=white)](<>)
[![crates.io](https://img.shields.io/crates/v/disaity?style=flat-square&color=D4AF37&logo=rust)](https://crates.io/crates/disaity)
[![docs.rs](https://img.shields.io/docsrs/disaity?style=flat-square&logo=docsdotrs)](https://docs.rs/disaity)
[![Rust Version](https://img.shields.io/badge/rust-latest-orange?logo=rust&logoColor=orange)](https://www.rust-lang.org)

</div>

<br/>

Disaity builds Discord bots that play music in voice channels, pre build features,
subscrible to events and send you back in character.

> [!NOTE]
> Disaity status is currently experimental, It can uses many commands and features
> but some methods may changes in the future.

## Getting started

```bash
cargo new bot && cd bot
cargo add disaity tokio -F tokio/macros,tokio/rt-multi-thread
```

Create a `.env` file at the root for your project or add it in your system,
You can use the default key names or custom,
then run `cargo run -- --install bin` once to fetch external binaries
if the required bin not found on the system or insallted on bin directory
the bot will panic,
refer to [binaries](https://github.com/AhmedEhab-SG/disaity/tree/main/BINARIES.md)
for more info, then `cargo run`.

Some features requires additional enviroment key;

- `chat` features uses ai models needs provider url.
- `subscription` features uses SQLite just need a path folder.

refer to [.env.example](https://github.com/AhmedEhab-SG/disaity/tree/main/.env.example)
to follow the default env key, you can still pass you custom key while building
the client instead of the default keys.

## Example

Basic music bot looks like:

```rust
use disaity::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Binaries::ensure().await?;

    let data = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_info(Info::new().with_prefix("!"))
                .with_persona(Persona::new(Preset::Emilia))
                .build(),
        )
        .build()
        .await?;

    Client::new()
        .with_feature(Commands::music())
        .with_feature(Commands::other())
        .with_data(data)
        .run()
        .await
}
```

Or test an [example](crates/disaity/examples/basic.rs)

```bash
git clone https://github.com/AhmedEhab-SG/disaity
cd disaity
cp .env.example .env    # then fill in CLIENT_TOKEN

cargo run --example basic -- --install bin    # fills bin/, no token needed
cargo run --example basic
```

View [full examples](crates/disaity/examples)
on how to setup and use disiaty with its builtin features to make you custom charature bot.

> [!NOTE]
> Make sure to add to ensure the bin are loaded or it will fail on runetime
> when trying to use the target binaray.
>
> ```rust
> Binaries::ensure().await?; // looks in this order to find the bin
> ```

## Default Features

**Playing music from any provider.** Playback goes through `yt-dlp` and
FFmpeg, YouTube, SoundCloud and Spotify defaults `yt-dlp`.

**Has it's own personality.** Personas are TOML, a system prompt for the AI;
point `Persona::from_file_over` at your own file to override any part or crate new.

**Chat and asists you** Talks to any ai using the persona's as a character.
The ai could excute what you need, It needs `AI_URL` register `Commands::chat()`

**Manage, handling server for you** Most of the server utils and customization.

**Subscriptions events.** A persistent tasks keeps running on background,
It needs `DB_PATH` to save it's options.

## Building

### Unix

```bash
# Head over to https://rust-lang.org and install `rustup` or
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install toolchain stable
rustup toolchain install stable
```

C compiler toolchain, and CMake is needed too.

#### Linux

```bash
sudo apt install build-essential cmake          # Debian/Ubuntu

# sudo dnf install @development-tools cmake     # Fedora
# sudo pacman -S base-devel cmake               # Arch
```

#### macOS

```bash
# clang: linker and C compiler
xcode-select --install

# install cmake
brew install cmake
```

### Windows

#### Install Rust and CMake

```powershell
winget install -e --id Rustlang.Rustup

# install cmake
winget install -e --id Kitware.CMake
```

On windows there is two options using VisualSudio build tool or gnu

#### **MSVC** — Rustup's default toolchain:

```powershell
# install msvc default on windows
rustup toolchain install stable
rustup default stable-x86_64-pc-windows-msvc

# install VS build tools
winget install --id Microsoft.VisualStudio.2022.BuildTools `
  --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

```

#### **MinGW** — Using gnu toolchain:

```powershell
# install toolchain gnu and default it
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

# isntall MinGW-w64 GCC (gcc, as, ld, mingw32-make)
winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT
```

### Build

Open a new shell after installing then build

```bash
cargo build
```
