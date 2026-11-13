use std::{
    env,
    path::{self, Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};

use super::consts::HOME;

/// If its global installed or the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Caller {
    Global,
    Client,
}

impl Caller {
    pub(crate) fn home(self, path: Option<&Path>) -> Result<PathBuf> {
        match (self, path) {
            (Self::Global, Some(path)) => {
                ensure!(path.is_dir(), "{} is not a folder", path.display());

                Ok(path::absolute(path)?)
            }

            (Self::Global, None) => BinUtils::env_home()
                .with_context(|| format!("need specific path use --path <DIR>, or set {HOME}")),

            (Self::Client, None) => BinUtils::env_home().map_or_else(BinUtils::exe_dir, Ok),

            (Self::Client, Some(_)) => {
                bail!("bin has fixed place at root directory, drop --path or set {HOME}")
            }
        }
    }
}

pub(super) struct BinUtils;

impl BinUtils {
    /// `DISAITY_HOME`, when it's set to something.
    fn env_home() -> Option<PathBuf> {
        env::var_os(HOME)
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
    }

    /// The folder this program's exe is in.
    fn exe_dir() -> Result<PathBuf> {
        let exe = env::current_exe().context("can't find where this program lives")?;

        exe.parent()
            .map(Path::to_path_buf)
            .with_context(|| format!("{} has no parent folder: set {HOME}", exe.display()))
    }

    #[cfg(unix)]
    pub(super) fn make_executable(path: &Path) -> Result<()> {
        use std::{fs, os::unix::fs::PermissionsExt};

        let mut perms = fs::metadata(path)?.permissions();

        perms.set_mode(perms.mode() | 0o111);

        fs::set_permissions(path, perms)?;

        Ok(())
    }

    #[cfg(not(unix))]
    pub(super) fn make_executable(_path: &Path) -> Result<()> {
        Ok(())
    }
}
