use std::{fs, io::Write, path::Path};

use crate::errors::Error;

use super::{archive, spec::BinarySpec, utils::BinUtils};

/// Fetch this platform's build of `spec` and leave it runnable at `path`,
/// creating the directories above it.
pub(super) async fn download(spec: &BinarySpec, path: &Path) -> Result<(), Error> {
    let url = spec.download_url()?;
    println!("⬇️ {url}");

    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)?;

    let staged = parent.join(format!("{}.part", spec.name));

    let result = match stream(&url, &staged).await {
        Ok(()) if archive::is_archive(&url) => archive::unpack(&staged, spec, path),
        Ok(()) => fs::rename(&staged, path).map_err(Into::into),
        Err(error) => Err(error),
    };

    let _ = fs::remove_file(&staged);
    result?;

    BinUtils::make_executable(path)?;
    println!("✅ {} → {}", spec.name, path.display());

    Ok(())
}

/// Write the response to disk as it arrives, so an ffmpeg archive never has to
/// fit in memory.
async fn stream(url: &str, path: &Path) -> Result<(), Error> {
    let mut response = reqwest::get(url).await?.error_for_status()?;
    let mut file = fs::File::create(path)?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }
    file.sync_all()?;

    Ok(())
}
