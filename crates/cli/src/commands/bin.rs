use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

use crate::{
    BinarySpec, FFMPEG, YTDLP,
    binaries::{ALL_BIN, Install, Mode},
};

#[derive(Debug, Subcommand)]
pub(crate) enum BinAction {
    #[command(short_flag = 'i', long_flag = "install")]
    Install(Target),

    #[command(short_flag = 'u', long_flag = "update")]
    Update(Target),
}

impl BinAction {
    pub(crate) async fn run(&self) -> Result<()> {
        match self {
            Self::Install(target) => Install(Mode::Get).run(&target.specs()).await,
            Self::Update(target) => Install(Mode::Update).run(&target.specs()).await,
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct Target {
    #[arg(value_enum)]
    names: Vec<Name>,
}

impl Target {
    fn specs(&self) -> Vec<&'static BinarySpec> {
        let all = self.names.is_empty() || self.names.contains(&Name::ALL);

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
    ALL,
    Ffmpeg,
    YtDlp,
}

impl Name {
    fn specs(self) -> Option<&'static BinarySpec> {
        match self {
            Self::ALL => None,
            Self::Ffmpeg => Some(&FFMPEG),
            Self::YtDlp => Some(&YTDLP),
        }
    }
}
