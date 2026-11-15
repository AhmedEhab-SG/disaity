use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};
use strum::VariantArray;

use disaity_config::Bin;

use super::super::{
    Binaries, BinarySpec,
    binaries::{Caller, Install, Mode},
};

#[derive(Debug, Subcommand)]
pub(crate) enum BinAction {
    #[command(short_flag = 'i', long_flag = "install")]
    Install(Target),

    #[command(short_flag = 'u', long_flag = "update")]
    Update(Target),
}

impl BinAction {
    pub(crate) async fn run(&self, caller: Caller) -> Result<()> {
        let (mode, target) = match self {
            Self::Install(target) => (Mode::Get, target),
            Self::Update(target) => (Mode::Update, target),
        };

        let home = caller.home(target.path.as_deref())?;

        Install(mode).run(&target.specs(), &home).await
    }
}

#[derive(Debug, Args)]
pub(crate) struct Target {
    #[arg(value_enum)]
    names: Vec<Bin>,

    #[arg(short, long, value_name = "DIR")]
    path: Option<PathBuf>,
}

impl Target {
    fn specs(&self) -> Vec<&'static BinarySpec> {
        let bins = if self.names.is_empty() {
            Bin::VARIANTS
        } else {
            &self.names
        };

        bins.iter().map(|&bin| Binaries::spec(bin)).collect()
    }
}
