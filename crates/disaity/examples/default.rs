//! Includes all the default features enabled refer to env docs to run it and install bin.
//!
//! ```sh
//! cp .env.example .env                        # then fill all needed tokens
//! cargo run --example default -- --install bin  # once, to fetch yt-dlp and ffmpeg
//! cargo run --example default
//! ```

use disaity::{PrayerSubscription, StatusHandler, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
    Binaries::ensure().await?;

    Client::new()
        .with_tracing()
        .with_feature(Commands::all())
        .with_handler(StatusHandler)
        .with_subscription(PrayerSubscription)
        .run()
        .await
}
