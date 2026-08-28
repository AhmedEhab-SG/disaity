use std::{env, path::PathBuf};

use crate::errors::Error;

use super::consts::BIN_DIR;

/// One published file, matched against [`env::consts::OS`] and
/// [`env::consts::ARCH`].
pub struct Download {
    os: &'static str,
    arch: &'static str,
    file: &'static str,
}

impl Download {
    pub(super) const fn new(os: &'static str, arch: &'static str, file: &'static str) -> Self {
        Self { os, arch, file }
    }

    fn is_host(&self) -> bool {
        self.os == env::consts::OS && self.arch == env::consts::ARCH
    }
}

/// An external program the framework knows how to find and how to fetch.
pub struct BinarySpec {
    pub name: &'static str,
    pub segments: &'static [&'static str],
    pub override_var: &'static str,
    pub(super) base_url: &'static str,
    pub(super) downloads: &'static [Download],
}

impl BinarySpec {
    /// `ffmpeg`, or `ffmpeg.exe` where that is what it is called.
    pub(super) fn file_name(&self) -> String {
        format!("{}{}", self.name, env::consts::EXE_SUFFIX)
    }

    /// Where our copy sits inside any root.
    pub(super) fn relative_path(&self) -> PathBuf {
        let mut path = PathBuf::from(BIN_DIR);
        path.extend(self.segments);
        path.push(self.file_name());
        path
    }

    pub(super) fn download_url(&self) -> Result<String, Error> {
        self.downloads
            .iter()
            .find(|download| download.is_host())
            .map(|download| format!("{}/{}", self.base_url, download.file))
            .ok_or_else(|| {
                format!(
                    "no {} build is published for {}-{}: install one yourself and point {} at it",
                    self.name,
                    env::consts::OS,
                    env::consts::ARCH,
                    self.override_var
                )
                .into()
            })
    }
}
