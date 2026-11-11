use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use tokio::sync::OnceCell;

use super::{bin::BinAction, completions::Completions, logs::Logs};

/// core cli can run from bot or framework
#[derive(Debug, Parser)]
#[command(name = "disaity", version)]
pub struct Interface {
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// Every command can be used in in `commands/`.
#[derive(Debug, Subcommand)]
pub enum Action {
    // Install or update bin files
    #[command(subcommand)]
    Bin(BinAction),

    /// Print tab completion for your shell
    Completions(Completions),
}

/// single instance
static DONE: OnceCell<Result<bool, String>> = OnceCell::const_new();

impl Interface {
    pub async fn run(self) -> Result<bool> {
        let Some(action) = self.action else {
            return Ok(false);
        };

        Logs::init();

        match action {
            Action::Bin(bin) => bin.run().await?,
            Action::Completions(completions) => completions.run(),
        }

        Ok(true)
    }

    pub async fn run_once() -> Result<bool> {
        DONE.get_or_init(|| async { Self::parse().run().await.map_err(|err| format!("{err:#}")) })
            .await
            .clone()
            .map_err(anyhow::Error::msg)
    }
}
