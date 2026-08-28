use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::errors::Error;

use super::consts::{BIN_DIR, HOME_VAR};

pub(super) struct BinUtils;

impl BinUtils {
    /// Directories that may hold a `bin/` tree, most specific first.
    ///
    /// `CARGO_MANIFEST_DIR` is set by `cargo run` at runtime and names the crate
    /// root of the package being run; its absence is what distinguishes a shipped
    /// binary from a development one.
    pub(super) fn roots() -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Some(home) = env::var_os(HOME_VAR) {
            roots.push(PathBuf::from(home));
        }

        if let Some(manifest) = env::var_os("CARGO_MANIFEST_DIR") {
            roots.push(PathBuf::from(manifest));
        }

        if let Ok(exe) = env::current_exe()
            && let Some(dir) = exe.parent()
        {
            roots.push(dir.to_path_buf());
        }

        if let Some(data) = Self::data_dir() {
            roots.push(data.join("disaity"));
        }

        roots
    }

    /// The first root a `bin/` tree can actually be created in.
    pub(super) fn install_root() -> Result<PathBuf, Error> {
        Self::roots()
            .into_iter()
            .find(|root| fs::create_dir_all(root.join(BIN_DIR)).is_ok())
            .ok_or_else(|| {
                format!(
                    "no writable location for `{BIN_DIR}/`: set {HOME_VAR} to a directory you own"
                )
                .into()
            })
    }

    fn data_dir() -> Option<PathBuf> {
        if cfg!(windows) {
            return env::var_os("LOCALAPPDATA").map(PathBuf::from);
        }

        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    }

    /// Minimal `which`: the first `PATH` entry holding an executable by this name.
    pub(super) fn which(file_name: &str) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;

        env::split_paths(&path)
            .map(|dir| dir.join(file_name))
            .find(|candidate| Self::is_executable(candidate))
    }

    fn is_executable(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::metadata(path)
                .map(|meta| meta.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }

        #[cfg(not(unix))]
        true
    }

    /// Add the execute bits a download arrives without, leaving the rest of the
    /// mode alone so a restrictive umask is respected.
    #[cfg(unix)]
    pub(super) fn make_executable(path: &Path) -> Result<(), Error> {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms)?;

        Ok(())
    }

    #[cfg(not(unix))]
    pub(super) fn make_executable(_path: &Path) -> Result<(), Error> {
        Ok(())
    }
}
