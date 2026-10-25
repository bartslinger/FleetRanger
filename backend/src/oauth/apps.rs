//! Trusted apps, which an admin adds on the Apps page: they skip the consent
//! screen, but still act only as their user. See "Apps" in
//! docs/architecture.md.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::{self, TrustedApp},
    routes::redirect_uri_allowed,
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Permission},
};

#[derive(Deserialize)]
pub(super) struct AppInput {
    name: String,
    redirect_uris: Vec<String>,
}

impl AppInput {
    /// The name trimmed, and return addresses as apps may register them.
    fn checked(&self) -> ApiResult<(&str, Vec<String>)> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(ApiError::Invalid(
                "Give the app a name of at most 100 characters".to_owned(),
            ));
        }
        let uris: Vec<String> = self
            .redirect_uris
            .iter()
            .map(|uri| uri.trim().to_owned())
            .filter(|uri| !uri.is_empty())
            .collect();
        if uris.is_empty() || uris.len() > 10 {
            return Err(ApiError::Invalid(
                "Give between 1 and 10 return addresses".to_owned(),
            ));
        }
        if let Some(uri) = uris.iter().find(|uri| !redirect_uri_allowed(uri)) {
            return Err(ApiError::Invalid(format!(
                "{uri} isn’t a valid return address: use HTTPS, or HTTP on this computer"
            )));
        }
        Ok((name, uris))
    }
}

pub(super) async fn list(
    actor: Actor,
    State(db): State<PgPool>,
) -> ApiResult<Json<Vec<TrustedApp>>> {
    policy::require(&actor, Permission::AppsManage, None)?;
    Ok(Json(db::trusted_apps(&db).await?))
}

#[derive(Serialize)]
pub(super) struct Added {
    id: Uuid,
}

pub(super) async fn add(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<AppInput>,
) -> ApiResult<(StatusCode, Json<Added>)> {
    policy::require(&actor, Permission::AppsManage, None)?;
    let (name, uris) = input.checked()?;
    let id = db::add_trusted(&db, name, &uris).await?;
    tracing::info!(%id, name, by = %actor.user_id, "trusted app added");
    Ok((StatusCode::CREATED, Json(Added { id })))
}

pub(super) async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<AppInput>,
) -> ApiResult<StatusCode> {
    policy::require(&actor, Permission::AppsManage, None)?;
    let (name, uris) = input.checked()?;
    if !db::update_trusted(&db, id, name, &uris).await? {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Removes an app: its users' tokens stop working at once.
pub(super) async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    policy::require(&actor, Permission::AppsManage, None)?;
    if !db::remove_trusted(&db, id).await? {
        return Err(ApiError::NotFound);
    }
    tracing::info!(%id, by = %actor.user_id, "trusted app removed");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::{
        db::{Code, Tokens},
        sha256,
    };

    /// Clean-up keeps a trusted app without users, as it removes registered
    /// ones; removing a trusted app ends its users' access at once.
    #[sqlx::test]
    async fn removing_a_trusted_app_ends_its_tokens(db: PgPool) {
        let user: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        sqlx::query("insert into logins (user_id, email) values ($1, 'a@example.org')")
            .bind(user)
            .execute(&db)
            .await
            .unwrap();
        let uris = ["https://app.example.org/callback".to_owned()];
        let app = db::add_trusted(&db, "Test app", &uris).await.unwrap();
        sqlx::query("update oauth_clients set created_at = now() - interval '2 days'")
            .execute(&db)
            .await
            .unwrap();
        db::clean_up(&db, 30).await.unwrap();
        assert!(
            db::client(&db, app)
                .await
                .unwrap()
                .is_some_and(|c| c.trusted)
        );

        let code = Code {
            client_id: app,
            user_id: user,
            redirect_uri: uris[0].clone(),
            code_challenge: String::new(),
            scopes: vec!["sessions:list".to_owned()],
            resource: "https://fleet.example.org/api/v1".to_owned(),
        };
        let tokens = Tokens {
            access_sha256: &sha256("access"),
            refresh_sha256: &sha256("refresh"),
            access_seconds: 60.0,
        };
        db::create_grant(&db, &code, &tokens).await.unwrap();
        assert!(
            db::use_access(&db, &sha256("access"))
                .await
                .unwrap()
                .is_some()
        );

        assert!(db::remove_trusted(&db, app).await.unwrap());
        assert!(
            db::use_access(&db, &sha256("access"))
                .await
                .unwrap()
                .is_none()
        );
    }
}
