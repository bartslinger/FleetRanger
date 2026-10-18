//! Account API, for admins.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{db::Account, service};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/accounts", get(list).post(create))
        .route("/api/accounts/{id}", get(show).put(update))
        .route("/api/accounts/{id}/disable", post(disable))
        .route("/api/accounts/{id}/enable", post(enable))
}

#[derive(Deserialize)]
struct NewAccount {
    email: String,
    role: String,
}

#[derive(Deserialize)]
struct AccountUpdate {
    role: String,
}

async fn list(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Account>>> {
    Ok(Json(service::list(&db, &actor).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Account>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(body): Json<NewAccount>,
) -> ApiResult<Json<Account>> {
    Ok(Json(
        service::create(&db, &actor, &body.email, &body.role).await?,
    ))
}

async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<AccountUpdate>,
) -> ApiResult<Json<Account>> {
    Ok(Json(service::set_role(&db, &actor, id, &body.role).await?))
}

async fn disable(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Account>> {
    Ok(Json(service::set_disabled(&db, &actor, id, true).await?))
}

async fn enable(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Account>> {
    Ok(Json(service::set_disabled(&db, &actor, id, false).await?))
}
