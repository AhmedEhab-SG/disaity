//! The smallest useful bot: music, chat and the default persona.
//!
//! ```sh
//! cp .env.example .env                        # then fill in CLIENT_TOKEN
//! cargo run --example basic -- --install bin  # once, to fetch yt-dlp and ffmpeg
//! cargo run --example basic
//! ```
//!
//! `Binaries::ensure()` only locates `yt-dlp` and `ffmpeg`; `--install bin`
//! puts them in a `bin/` directory next to this crate's `Cargo.toml`, and
//! `--update bin` replaces them with the current upstream builds.

use disaity::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Binaries::ensure().await?;

    let data = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_persona(Persona::new(Preset::Emilia))
                .build(),
        )
        .build()
        .await?;

    Client::new(&data.config.env.client_token)
        .with_prefix(&data.config.info.prefix)
        .with_feature(Commands::music())
        .with_feature(Commands::chat())
        .with_feature(Commands::other())
        .with_data(data)
        .run()
        .await
}
