use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

use crate::{
    BinarySpec, FFMPEG, YTDLP,
    binaries::{ALL_BIN, Caller, Install, Mode},
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
    names: Vec<Name>,

    #[arg(short, long, value_name = "DIR")]
    path: Option<PathBuf>,
}

impl Target {
    fn specs(&self) -> Vec<&'static BinarySpec> {
        let all = self.names.is_empty() || self.names.contains(&Name::All);

        ALL_BIN
            .iter()
            .copied()
            .filter(|spec| {
                all || self
                    .names
                    .iter()
                    .filter_map(|name| name.specs())
                    .any(|picked| picked.name == spec.name)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Name {
    All,
    Ffmpeg,
    YtDlp,
}

impl Name {
    fn specs(self) -> Option<&'static BinarySpec> {
        match self {
            Self::All => None,
            Self::Ffmpeg => Some(&FFMPEG),
            Self::YtDlp => Some(&YTDLP),
        }
    }
}
