//! The smallest useful bot: music, chat and the default persona.
//!
//! ```sh
//! cp .env.example .env                        # then fill in CLIENT_TOKEN
//! cargo run --example basic
//! ```

use disaity::{prelude::*, tokio};

#[tokio::main(crate = "disaity::tokio")]
async fn main() -> Result<(), Error> {
    // default prefix is `!`
    // run your client and ping it
    Client::new()
        .with_feature(Commands::new().with(CommandId::Ping))
        .run()
        .await
}
