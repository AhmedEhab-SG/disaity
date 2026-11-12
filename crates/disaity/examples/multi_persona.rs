//! Run multi clients with its own token, prefix and database.
//!
//! ```sh
//! cargo run --example multi_persona
//! ```
//!
//! Needs `CLIENT_TOKEN` and `CLIENT_TOKEN_ALT` in your `.env`. Set `LOG_LEVEL`
//! `with_tracing()`, is need to show logs.
//! `cargo run --example multi_persona -- -bi` # to install all bins

use disaity::{FreeGamesSubscription, StatusHandler, WelcomeHandler, prelude::*, tokio};

#[tokio::main(crate = "disaity::tokio")]
async fn main() -> Result<(), Error> {
    // default prefix is `!` and current default persona
    let client = Client::new()
        .with_tracing()
        .with_feature(Commands::music())
        .with_feature(Commands::chat())
        .with_feature(Commands::other())
        .with_handler(WelcomeHandler)
        .with_handler(StatusHandler)
        .with_subscription(FreeGamesSubscription);

    let data = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_env(Env::new().with_client_token("CLIENT_TOKEN_ALT"))
                .with_info(Info::new().with_prefix("-"))
                .with_persona(Persona::new(Preset::Rem))
                .build(),
        )
        .build()
        .await?;

    let client_alt = Client::new()
        .with_tracing()
        .with_feature(Commands::manage())
        .with_feature(Commands::chat())
        .with_feature(Commands::other())
        .with_handler(StatusHandler)
        .with_data(data);

    let handle_emilia = tokio::spawn(client.run());
    let handle_rem = tokio::spawn(client_alt.run());

    let (thread_emilia, thread_rem) = tokio::try_join!(handle_emilia, handle_rem)?;
    thread_emilia?;
    thread_rem?;

    Ok(())
}
