//! The organisation's name and settings, for its admins (user's decision,
//! 2026-11-13: admins change all of them, as the operator can), and deleting
//! it with everything in it (user's decision, 2026-11-13).

use axum::{Json, Router, extract::FromRef, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::Connection as _;

use crate::{
    auth::Actor,
    db::{Db, System},
    error::{self, ApiError, ApiResult},
    ingest::store::LogStore,
    oauth::AppsOn,
    policy::{self, Permission},
};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
    AppsOn: FromRef<S>,
    LogStore: FromRef<S>,
{
    Router::new().route("/api/organisation", get(show).put(save).delete(remove))
}

#[derive(Serialize)]
struct Organisation {
    name: String,
    time_zone: String,
    session_gap_minutes: i32,
    mcp: bool,
    /// Whether the install offers MCP at all (`FLEETRANGER_MCP`): only then
    /// is the organisation's own switch worth showing.
    mcp_offered: bool,
    /// The time zones to choose from.
    time_zones: Vec<String>,
}

#[derive(Deserialize)]
struct Input {
    name: String,
    time_zone: String,
    session_gap_minutes: i32,
    /// Left as it is when not given, as when the install doesn't offer MCP.
    mcp: Option<bool>,
}

async fn show(
    actor: Actor,
    State(apps): State<AppsOn>,
    mut db: Db,
) -> ApiResult<Json<Organisation>> {
    policy::require(&actor, Permission::OrganisationManage, None)?;
    Ok(Json(read(&mut db, apps).await?))
}

async fn save(
    actor: Actor,
    State(apps): State<AppsOn>,
    mut db: Db,
    Json(input): Json<Input>,
) -> ApiResult<Json<Organisation>> {
    policy::require(&actor, Permission::OrganisationManage, None)?;
    let name = error::name(&input.name)?;
    if !super::known_time_zone(&mut *db, &input.time_zone).await? {
        return Err(ApiError::Invalid(
            "Choose a time zone from the list.".to_owned(),
        ));
    }
    if input.session_gap_minutes < 1 {
        return Err(ApiError::Invalid(
            "The session gap is at least 1 minute.".to_owned(),
        ));
    }
    let mcp = input.mcp.filter(|_| apps.mcp);
    let organisation_id = db.organisation_id;
    let mut tx = (*db).begin().await?;
    // Row-level security limits both to the request's own organisation.
    sqlx::query!(
        "update organisations set name = $2 where id = $1",
        organisation_id,
        name,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "update organisation_settings
         set time_zone = $2, session_gap_minutes = $3, mcp = coalesce($4, mcp)
         where organisation_id = $1",
        organisation_id,
        input.time_zone,
        input.session_gap_minutes,
        mcp,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(read(&mut db, apps).await?))
}

async fn read(db: &mut Db, apps: AppsOn) -> sqlx::Result<Organisation> {
    let settings = super::settings(db).await?;
    Ok(Organisation {
        name: settings.name,
        time_zone: settings.time_zone,
        session_gap_minutes: settings.session_gap.0,
        mcp: settings.mcp,
        mcp_offered: apps.mcp,
        time_zones: super::time_zones(db).await?,
    })
}

#[derive(Deserialize)]
struct Removal {
    /// The organisation's name, typed to confirm.
    name: String,
}

/// Deletes the organisation with everything in it, at once. Its logins go
/// with it, the admin's among them, so this request's login session ends.
async fn remove(
    actor: Actor,
    State(system): State<System>,
    State(store): State<LogStore>,
    mut db: Db,
    Json(removal): Json<Removal>,
) -> ApiResult<StatusCode> {
    policy::require(&actor, Permission::OrganisationManage, None)?;
    let name = super::settings(&mut db).await?.name;
    if removal.name.trim() != name {
        return Err(ApiError::Invalid(
            "Type the organisation's name exactly.".to_owned(),
        ));
    }
    let organisation_id = db.organisation_id;
    drop(db);
    super::delete::rows(&system, organisation_id, &name).await?;
    tracing::info!(
        %organisation_id,
        by = %actor.user_id,
        impersonated_by = ?actor.impersonated_by.as_ref().map(|admin| admin.user_id),
        "organisation deleted"
    );
    // The rows are gone, the deletion done: the files follow on their own,
    // so a closed page or a slow bucket doesn't stop them halfway.
    tokio::spawn(async move {
        match super::delete::files(&store, organisation_id).await {
            Ok(files) => tracing::info!(%organisation_id, files, "raw logs deleted"),
            Err(error) => tracing::error!("{error:#}"),
        }
    });
    Ok(StatusCode::NO_CONTENT)
}
