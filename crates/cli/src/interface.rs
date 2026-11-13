use anyhow::{Error, Ok, Result};
use clap::Parser;
use tokio::sync::OnceCell;

use super::{Action, binaries::Caller, logger::Logger};

/// core cli can run from bot or framework
#[derive(Debug, Parser)]
#[command(name = "disaity", version)]
pub struct Interface {
    #[command(subcommand)]
    action: Option<Action>,
}

/// single instance
static DONE: OnceCell<Result<bool, String>> = OnceCell::const_new();

impl Interface {
    pub async fn run(self) -> Result<bool> {
        self.exec(Caller::Global).await
    }

    pub async fn run_once() -> Result<bool> {
        DONE.get_or_init(|| async {
            Self::parse()
                .exec(Caller::Client)
                .await
                .map_err(|err| format!("{err:#}"))
        })
        .await
        .clone()
        .map_err(Error::msg)
    }

    async fn exec(self, caller: Caller) -> Result<bool> {
        let Some(action) = self.action else {
            return Ok(false);
        };

        Logger::init();

        match action {
            Action::Bin(bin) => bin.run(caller).await?,
            Action::Completions(completions) => completions.run(),
        }

        Ok(true)
    }
}
