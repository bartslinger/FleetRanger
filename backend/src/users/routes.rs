//! User API, for admins.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::User,
    service::{self, UserInput},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/users", get(list).post(create))
        .route("/api/users/{id}", get(show).put(update).delete(remove))
        .route("/api/users/{id}/archive", post(archive))
        .route("/api/users/{id}/restore", post(restore))
}

#[derive(Deserialize)]
struct ListQuery {
    /// Include archived users.
    #[serde(default)]
    archived: bool,
}

#[derive(Deserialize)]
struct Body {
    name: String,
    /// Roles on the whole organisation.
    #[serde(default)]
    roles: Vec<String>,
    email: Option<String>,
    /// Apps' roles on the whole organisation, by id; left out keeps them.
    app_roles: Option<Vec<Uuid>>,
}

impl From<Body> for UserInput {
    fn from(body: Body) -> Self {
        Self {
            name: body.name,
            roles: body.roles,
            email: body.email,
            app_roles: body.app_roles,
        }
    }
}

async fn list(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(service::list(&db, &actor, query.archived).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(body): Json<Body>,
) -> ApiResult<Json<User>> {
    Ok(Json(service::create(&db, &actor, &body.into()).await?))
}

async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<User>> {
    Ok(Json(service::update(&db, &actor, id, &body.into()).await?))
}

async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::delete(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn archive(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    Ok(Json(service::set_archived(&db, &actor, id, true).await?))
}

async fn restore(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    Ok(Json(service::set_archived(&db, &actor, id, false).await?))
}
