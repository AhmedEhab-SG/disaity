//! Includes all the default features enabled refer to env docs to run it and install bin.
//!
//! ```sh
//! cp .env.example .env                        # then fill all needed tokens
//! cargo run --example default -- -bi          # to fetch bins
//! cargo run --example default
//! ```

use disaity::{prelude::*, tokio};

#[tokio::main(crate = "disaity::tokio")]
async fn main() -> Result<(), Error> {
    Client::new()
        .with_tracing()
        .with_commands(Commands::all())
        .with_handlers(Handlers::all())
        .with_subscriptions(Subscriptions::all())
        .run()
        .await
}
