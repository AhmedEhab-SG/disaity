use std::{
    env, fmt,
    path::{Path, PathBuf},
};

use strum::Display;

use crate::errors::Error;

use super::{consts::BIN_DIR, spec::BinarySpec, utils::BinUtils};

/// Where a resolved binary came from, so a system copy shadowing an installed
/// one is visible at startup.
#[derive(Display, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(super) enum Source {
    Override,
    Bundled,
    System,
}

/// A located program, ready to hand to `Command::new`.
pub struct ResolvedBinary {
    spec: &'static BinarySpec,
    path: PathBuf,
    source: Source,
}

impl fmt::Display for ResolvedBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "✅ {} ({}) → {}",
            self.spec.name,
            self.source,
            self.path.display()
        )
    }
}

impl ResolvedBinary {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path as a string, for the APIs that take a program by name.
    pub fn program(&self) -> &str {
        self.path.to_str().unwrap_or(self.spec.name)
    }

    pub fn dir(&self) -> Option<&Path> {
        self.path.parent()
    }

    pub(super) fn source(&self) -> Source {
        self.source
    }

    /// Whether this is the copy `--install bin` put in `bin/`, which is the
    /// only one an update may write over.
    pub(super) fn is_bundled(&self) -> bool {
        self.source == Source::Bundled
    }

    /// Search an override, then every root, then `PATH` — first hit wins, so a
    /// copy we installed beats one the system happens to have.
    pub(super) fn locate(spec: &'static BinarySpec) -> Result<Self, Error> {
        if let Some(path) = env::var_os(spec.override_var).map(PathBuf::from) {
            if path.is_file() {
                return Ok(Self::new(spec, path, Source::Override));
            }
            println!(
                "⚠️ {} is set but does not point at a file, ignoring it.",
                spec.override_var
            );
        }

        let relative = spec.relative_path();
        if let Some(path) = BinUtils::roots()
            .into_iter()
            .map(|root| root.join(&relative))
            .find(|path| path.is_file())
        {
            return Ok(Self::new(spec, path, Source::Bundled));
        }

        if let Some(path) = BinUtils::which(&spec.file_name()) {
            return Ok(Self::new(spec, path, Source::System));
        }

        Err(format!(
            "{} was not found: run with `--install {BIN_DIR}` to download it, \
             or point {} at your own copy",
            spec.name, spec.override_var
        )
        .into())
    }

    fn new(spec: &'static BinarySpec, path: PathBuf, source: Source) -> Self {
        Self { spec, path, source }
    }
}
