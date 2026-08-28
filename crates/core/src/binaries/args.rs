use std::env;

use super::consts::BIN_DIR;

#[derive(Clone, Copy)]
pub(super) enum Action {
    Install,
    Update,
}

impl Action {
    pub(super) fn parse() -> Option<Self> {
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            let action = match arg.as_str() {
                "--install" => Self::Install,
                "--update" => Self::Update,
                _ => continue,
            };

            if args.next().as_deref() == Some(BIN_DIR) {
                return Some(action);
            }
        }

        None
    }
}
