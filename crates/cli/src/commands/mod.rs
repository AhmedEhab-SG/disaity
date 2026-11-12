mod bin;
mod completions;

use clap::Subcommand;

pub(crate) use {bin::BinAction, completions::Completions};

/// Every command can be used in in `commands/`.
#[derive(Debug, Subcommand)]
pub(crate) enum Action {
    // Install or update bin files
    #[command(subcommand, short_flag = 'b', long_flag = "bin")]
    Bin(BinAction),

    /// Print tab completion for your shell
    Completions(Completions),
}
