use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::errors::Error;

use super::spec::BinarySpec;

/// Whether a URL names something that has to be opened before it is a program.
pub(super) fn is_archive(url: &str) -> bool {
    url.ends_with(".tar.xz") || url.ends_with(".zip")
}

/// Unpack `archive` into a scratch directory, move the one file we want to
/// `path`, and take the scratch directory back down either way.
pub(super) fn unpack(archive: &Path, spec: &BinarySpec, path: &Path) -> Result<(), Error> {
    let dir = archive.with_extension("unpacked");
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir)?;

    let result = extract(archive, &dir).and_then(|()| {
        let file_name = spec.file_name();
        let found = find(&dir, &file_name)
            .ok_or_else(|| format!("{file_name} was not inside {}", archive.display()))?;

        fs::rename(found, path).map_err(Into::into)
    });

    fs::remove_dir_all(&dir).ok();

    result
}

/// `tar` reads both formats and ships with every Unix and with Windows since
/// 10 — cheaper than carrying two decompressors to run a command the user asks
/// for by hand. Everything is extracted, because GNU tar and the bsdtar on
/// Windows disagree about `--wildcards` and `--strip-components`.
fn extract(archive: &Path, into: &Path) -> Result<(), Error> {
    let status = Command::new("tar")
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(into)
        .status()
        .map_err(|error| {
            format!(
                "could not run `tar` to unpack {}: {error}",
                archive.display()
            )
        })?;

    if !status.success() {
        return Err(format!("tar exited with {status} unpacking {}", archive.display()).into());
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

    nested.iter().find_map(|dir| find(dir, name))
}
