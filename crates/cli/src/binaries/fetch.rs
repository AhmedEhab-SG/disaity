use std::{fs, io::Write, path::Path};

use anyhow::{Context, Result};
use indicatif::ProgressStyle;
use tracing::field::Empty;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use super::{archive::Archive, spec::BinarySpec, utils::BinUtils};

pub(super) struct Fetch<'a> {
    pub(super) path: &'a Path,
}

impl<'a> Fetch<'a> {
    pub(super) fn new(path: &'a Path) -> Self {
        Self { path }
    }

    /// Fetch this platform's build of `spec` and leave it runnable at `path`,
    /// creating the directories above it.
    pub(super) async fn download(&self, spec: &BinarySpec) -> Result<()> {
        let url = spec.download_url()?;

        tracing::info!("⬇️ {url}");

        let parent = self
            .path
            .parent()
            .with_context(|| format!("{} has no parent directory", self.path.display()))?;

        fs::create_dir_all(parent)?;

        let staged = parent.join(format!("{}.part", spec.name));

        let result = match self.stream(&url, &spec).await {
            Ok(()) if Archive::is_archive(&url) => Archive::new(&staged).unpack(spec, self.path),

            Ok(()) => fs::rename(&staged, self.path).map_err(Into::into),

            Err(error) => Err(error),
        };

        fs::remove_file(&staged).ok();

        result?;

        BinUtils::make_executable(self.path)?;

        tracing::info!("✅ {} → {}", spec.name, self.path.display());

        Ok(())
    }

    /// Write the response to disk as it arrives.
    /// The span is the progress bar: tracing-indicatif draws it on terminal only
    async fn stream(&self, url: &str, spec: &BinarySpec) -> Result<()> {
        let mut response = reqwest::get(url)
            .await
            .and_then(|res| res.error_for_status())
            .with_context(|| format!("couldn't download {}", spec.name))?;

        let mut file = fs::File::create(self.path)?;

        let bar = tracing::info_span!("download", indicatif.pb_show = Empty);

        bar.pb_set_message(spec.name);

        bar.pb_set_style(&ProgressStyle::with_template(
            "{msg:8} {wide_bar} {bytes}/{total_bytes} {eta}",
        )?);

        bar.pb_start();

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;

            bar.pb_inc(chunk.len() as u64);
        }

        file.sync_all()?;

        Ok(())
    }
}
