use std::path::Path;

use anyhow::Result;

use super::{
    fetch::Fetch,
    resolve::{ResolvedBinary, Source},
    spec::BinarySpec,
};

/// What `bin install` and `bin update` do, without the command line around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Get,
    Update,
}

pub(crate) struct Install(pub(crate) Mode);

impl Install {
    pub(crate) async fn run(&self, specs: &[&'static BinarySpec], home: &Path) -> Result<()> {
        for &spec in specs {
            match (ResolvedBinary::locate(spec, home), self.0) {
                (Ok(found), Mode::Get) if found.is_bundled() => {
                    tracing::info!("✅ {} is installed → {}", spec.name, found.path().display());
                }

                (Ok(found), Mode::Update) if found.is_bundled() => {
                    Fetch::new(found.path()).download(spec).await?;
                }

                (Ok(found), Mode::Update) => tracing::info!("⏭️ {}", Self::not_ours(spec, &found)),

                // skipped, the other binaries still update
                (Err(error), Mode::Update) => tracing::warn!("⏭️ {error}"),

                (_, Mode::Get) => {
                    Fetch::new(&home.join(spec.relative_path()))
                        .download(spec)
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
             or run `bin install {}` for a copy disaity manages",
                spec.name,
                found.path().display(),
                spec.name
            ),
            _ => format!(
                "{} is the copy {} points at — update it yourself",
                spec.name, spec.override_var
            ),
        }
    }
}
