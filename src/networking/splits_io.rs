//! The `splits_io` module provides communication with
//! [splits.io](https://splits.io). The raw API is available via the `api`
//! submodule. Additional helpers for directly uploading and downloading Run
//! objects are available as well.

use crate::run::{parser::composite, saver};
use crate::{Run, Timer};
use snafu::ResultExt;
use std::io::Cursor;

pub use api::{run::UploadedRun, Client, Error as UploadError};
pub use splits_io_api as api;

/// Describes an error that happened when downloading a run from splits.io. This
/// may either be because the download itself had a problem or because the run
/// itself couldn't be parsed.
#[derive(Debug, snafu::Snafu)]
pub enum DownloadError {
    /// Failed to download the run.
    Download {
        /// The underlying download error.
        source: api::Error,
    },
    /// Failed to parse the run.
    Parse {
        /// The underlying parsing error.
        source: composite::Error,
    },
}

/// Asynchronously downloads a run from splits.io based on its splits.io ID. The
/// run automatically gets parsed into a Run object.
pub async fn download_run(
    client: &Client,
    id: &str,
) -> Result<composite::ParsedRun, DownloadError> {
    let bytes = api::run::download(client, id).await.context(Download)?;
    let bytes: &[u8] = &*bytes;
    composite::parse(Cursor::new(bytes), None, false).context(Parse)
}

/// Asynchronously uploads a run to splits.io. An object representing the ID of
/// the uploaded run and its claim token gets returned when the run was
/// successfully uploaded.
pub async fn upload_run(client: &Client, run: &Run) -> Result<UploadedRun, UploadError> {
    api::run::upload_lazy(client, |writer| saver::livesplit::save_run(run, writer)).await
}

/// Asynchronously uploads the run of the timer provided to splits.io. If there
/// is an attempt in progress, a copy that has been reset will be uploaded. An
/// object representing the ID of the uploaded run and its claim token gets
/// returned when the run was successfully uploaded.
pub async fn upload_timer(client: &Client, timer: &Timer) -> Result<UploadedRun, UploadError> {
    api::run::upload_lazy(client, |writer| saver::livesplit::save_timer(timer, writer)).await
}
