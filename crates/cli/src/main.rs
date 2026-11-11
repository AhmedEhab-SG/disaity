//! The `disaity` command, for installing without starting a bot.
//!
//! ```sh
//! cargo run -p disaity-cli -- bin install
//! cargo install --path crates/cli     # then anywhere: disaity bin -u yt-dlp
//! ```

use clap::{CommandFactory, Parser};

use disaity_cli::Interface;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if !Interface::parse().run().await? {
        // no command: the help is the answer, not a log line
        Interface::command().print_help()?;
    }

    Ok(())
}
