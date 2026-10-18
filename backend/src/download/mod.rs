//! The controller app's download, public like the download page: one APK
//! file that the install's administrator puts on the server (Ansible),
//! served with this install's address written into it (see `apk`).

mod apk;

use std::path::PathBuf;

use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::header,
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use crate::error::{ApiError, ApiResult};

/// Where the APK is (`FLEETRANGER_ANDROID_APK`; it may not be there), and
/// the address it gets, e.g. `https://fleet.example.org`.
#[derive(Clone)]
pub struct AndroidApk {
    pub path: PathBuf,
    pub server: String,
}

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AndroidApk: FromRef<S>,
{
    Router::new()
        .route("/api/download", get(available))
        .route("/api/download/FleetRanger.apk", get(apk))
}

#[derive(Serialize)]
struct Available {
    /// Null when the install has no APK.
    android: Option<Apk>,
}

#[derive(Serialize)]
struct Apk {
    size_bytes: u64,
}

async fn available(State(apk): State<AndroidApk>) -> Json<Available> {
    let size = tokio::task::spawn_blocking(move || std::fs::metadata(&apk.path))
        .await
        .ok()
        .and_then(Result::ok)
        .filter(|metadata| metadata.is_file())
        .map(|metadata| Apk {
            size_bytes: metadata.len(),
        });
    Json(Available { android: size })
}

async fn apk(State(apk): State<AndroidApk>) -> ApiResult<impl IntoResponse> {
    let read = tokio::task::spawn_blocking(move || {
        std::fs::read(&apk.path).map(|bytes| {
            // Without the address the app still works: it asks for it.
            apk::with_server(&bytes, &apk.server).unwrap_or_else(|error| {
                tracing::warn!("couldn't write the address into the APK: {error:#}");
                bytes
            })
        })
    })
    .await
    .map_err(|error| ApiError::Internal(error.into()))?;
    let bytes = match read {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ApiError::NotFound);
        }
        Err(error) => {
            return Err(ApiError::Internal(
                anyhow::Error::from(error).context("reading the APK failed"),
            ));
        }
    };
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.android.package-archive",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"FleetRanger.apk\"",
            ),
            // A new version replaces the file under the same name.
            (header::CACHE_CONTROL, "no-cache"),
        ],
        bytes,
    ))
}
