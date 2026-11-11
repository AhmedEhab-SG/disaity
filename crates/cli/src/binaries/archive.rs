use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use super::spec::BinarySpec;

pub(super) struct Archive<'a> {
    path: &'a Path,
}

impl<'a> Archive<'a> {
    pub(super) fn new(path: &'a Path) -> Self {
        Self { path }
    }

    /// Whether a URL names something that has to be opened before it is a program.
    pub(super) fn is_archive(url: &str) -> bool {
        url.ends_with(".tar.xz") || url.ends_with(".zip")
    }

    /// Unpack `archive` into a scratch directory, move the one file we want to
    /// `path`, and take the scratch directory back down either way.
    pub(super) fn unpack(&self, spec: &BinarySpec, path: &Path) -> Result<()> {
        let dir = self.path.with_extension("unpacked");

        fs::remove_dir_all(&dir).ok();

        fs::create_dir_all(&dir)?;

        let result = self.extract(&dir).and_then(|()| {
            let file_name = spec.file_name();

            let found = Self::find(&dir, &file_name)
                .with_context(|| format!("{file_name} was not inside {}", self.path.display()))?;

            fs::rename(found, path).map_err(Into::into)
        });

        fs::remove_dir_all(&dir).ok();

        result
    }

    /// `tar` reads both formats and ships with every Unix and with Windows since
    /// 10 — cheaper than carrying two decompressors to run a command the user asks
    /// for by hand. Everything is extracted, because GNU tar and the bsdtar on
    /// Windows disagree about `--wildcards` and `--strip-components`.
    fn extract(&self, into: &Path) -> Result<()> {
        let status = Command::new("tar")
            .arg("-xf")
            .arg(self.path)
            .arg("-C")
            .arg(into)
            .status()
            .with_context(|| format!("could not run `tar` to unpack {}", self.path.display()))?;

        if !status.success() {
            bail!("tar exited with {status} unpacking {}", self.path.display());
        }

        Ok(())
    }

    /// Depth-first search for a file by name.
    fn find(dir: &Path, name: &str) -> Option<PathBuf> {
        let mut nested = Vec::new();

        for entry in fs::read_dir(dir).ok()?.flatten() {
            let path = entry.path();

            if path.is_dir() {
                nested.push(path);
            } else if path.file_name().is_some_and(|found| found == name) {
                return Some(path);
            }
        }

        nested.iter().find_map(|dir| Self::find(dir, name))
    }
}
