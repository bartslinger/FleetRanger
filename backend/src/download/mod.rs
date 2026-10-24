//! The controller app's download, public like the download page: one APK
//! file that the install's administrator puts on the server (Ansible),
//! served with this install's address written into it (see `apk`).

mod apk;

use std::path::PathBuf;

use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::header,
    middleware,
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
    // The release the controller updates to, in its API (see "Clients" in
    // docs/architecture.md); the download page keeps `/api/download`, whose
    // version fields android-v0.3.0 also reads.
    let v1 = Router::new()
        .route("/api/controller/v1/release", get(release))
        .route("/api/controller/v1/release/FleetRanger.apk", get(apk))
        .route_layer(middleware::from_fn(crate::pairing::v1_errors));
    Router::new()
        .route("/api/download", get(available))
        .route("/api/download/FleetRanger.apk", get(apk))
        .merge(v1)
}

#[derive(Serialize)]
struct Release {
    version_code: i64,
    version_name: String,
}

/// The release this install offers; none (404) without an APK, or without
/// its version, which leaves the app nothing to compare.
async fn release(State(apk): State<AndroidApk>) -> ApiResult<Json<Release>> {
    let Json(Available { android }) = available(State(apk)).await;
    match android {
        Some(Apk {
            version_code: Some(version_code),
            version_name: Some(version_name),
            ..
        }) => Ok(Json(Release {
            version_code,
            version_name,
        })),
        _ => Err(ApiError::NotFound),
    }
}

#[derive(Serialize)]
struct Available {
    /// Null when the install has no APK.
    android: Option<Apk>,
}

#[derive(Serialize)]
struct Apk {
    size_bytes: u64,
    /// From the build's metadata next to the APK (`FleetRanger.json` beside
    /// `FleetRanger.apk`); the app compares it with its own to offer updates.
    /// Null without that file.
    version_code: Option<i64>,
    version_name: Option<String>,
}

async fn available(State(apk): State<AndroidApk>) -> Json<Available> {
    let android = tokio::task::spawn_blocking(move || {
        let metadata = std::fs::metadata(&apk.path).ok().filter(|m| m.is_file())?;
        let version = std::fs::read(apk.path.with_extension("json"))
            .ok()
            .and_then(|json| serde_json::from_slice::<serde_json::Value>(&json).ok())
            .and_then(|json| json["elements"].get(0).cloned());
        Some(Apk {
            size_bytes: metadata.len(),
            version_code: version.as_ref().and_then(|v| v["versionCode"].as_i64()),
            version_name: version
                .as_ref()
                .and_then(|v| v["versionName"].as_str().map(str::to_owned)),
        })
    })
    .await
    .ok()
    .flatten();
    Json(Available { android })
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
