//! The `disaity` command
//!
//! ```sh
//! cargo run -p disaity-cli -- <cmd>               # in the repo: bin/ at its root
//! cargo install --path crates/cli                 # then name the bot's folder:
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
