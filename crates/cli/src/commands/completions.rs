use std::io;

use clap::{Args, CommandFactory};
use clap_complete::Shell;

use crate::Interface;

#[derive(Debug, Args)]
pub(crate) struct Completions {
    shell: Shell,
}

impl Completions {
    /// Writes the script to stdout, e.g. `disaity completions zsh > _disaity`.
    pub(crate) fn run(self) {
        let mut cli = Interface::command();
        let name = cli.get_name().to_owned();

        clap_complete::generate(self.shell, &mut cli, name, &mut io::stdout());
    }
}
