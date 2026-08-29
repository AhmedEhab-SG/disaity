//! Two personas from one process, each with its own token, prefix and database.
//!
//! ```sh
//! cargo run --example multi_persona
//! ```
//!
//! Needs `CLIENT_TOKEN` and `CLIENT_TOKEN_ALT` in your `.env`. Set `LOG_LEVEL`
//! there too — `debug` or `trace` — to watch both personas on stderr.

use disaity::{PrayerModule, StatusHandler, prelude::*};

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

    let client = Client::new()
        .with_tracing()
        .with_feature(Commands::music())
        .with_feature(Commands::other())
        .with_feature(Commands::chat())
        .with_handler(StatusHandler)
        .with_subscription(PrayerModule)
        .with_data(data);

    let data_alt = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_env(Env::new().with_client_token("CLIENT_TOKEN_ALT"))
                .with_info(Info::new().with_prefix("~"))
                .with_persona(Persona::new(Preset::Rem))
                .build(),
        )
        .build()
        .await?;

    let client_alt = Client::new()
        .with_tracing()
        .with_feature(Commands::music())
        .with_feature(Commands::other())
        .with_feature(Commands::chat())
        .with_handler(StatusHandler)
        .with_data(data_alt);

    let handle_emilia = tokio::spawn(client.run());
    let handle_rem = tokio::spawn(client_alt.run());

    let (thread_emilia, thread_rem) = tokio::try_join!(handle_emilia, handle_rem)?;
    thread_emilia?;
    thread_rem?;

    Ok(())
}
