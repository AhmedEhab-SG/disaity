use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::consts::HOME;

pub(super) struct BinUtils;

impl BinUtils {
    /// The one directory that holds `bin/`: `DISAITY_HOME`, or this user's data
    /// directory. Installs go here and the search looks here, so the tool and
    /// every bot always agree. Under cargo, `.cargo/config.toml` sets
    /// `DISAITY_HOME` to the repo root.
    pub(super) fn home() -> Result<PathBuf> {
        if let Some(home) = env::var_os(HOME) {
            return Ok(PathBuf::from(home));
        }

        dirs::data_local_dir()
            .map(|dir| dir.join("disaity"))
            .with_context(|| format!("this system has not data directory: set {HOME}"))
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
