use crate::errors::Error;

use super::{
    args::Action,
    consts::{ALL_BIN, BIN_DIR},
    fetch,
    resolve::{ResolvedBinary, Source},
    spec::BinarySpec,
    utils::BinUtils,
};

pub(super) async fn run(action: Action) -> Result<(), Error> {
    for &spec in ALL_BIN {
        match (ResolvedBinary::locate(spec), action) {
            (Ok(found), Action::Install) if found.is_bundled() => {
                println!("✅ {} is installed → {}", spec.name, found.path().display());
            }

            (Ok(found), Action::Update) if found.is_bundled() => {
                fetch::download(spec, found.path()).await?;
            }

            (Ok(found), Action::Update) => println!("⏭️ {}", not_ours(spec, &found)),

            (Err(error), Action::Update) => println!("⏭️ {error}"),

            (_, Action::Install) => {
                fetch::download(spec, &BinUtils::install_root()?.join(spec.relative_path()))
                    .await?;
            }
        }
    }

    Ok(())
}

fn not_ours(spec: &BinarySpec, found: &ResolvedBinary) -> String {
    match found.source() {
        Source::System => format!(
            "{} is a system install at {} — update it with your package manager, \
             or run `--install {BIN_DIR}` for a copy disaity manages",
            spec.name,
            found.path().display()
        ),
        _ => format!(
            "{} is the copy {} points at — update it yourself",
            spec.name, spec.override_var
        ),
    }
}
