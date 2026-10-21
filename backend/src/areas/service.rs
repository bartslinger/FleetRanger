use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Area};
use super::geometry;
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy,
};

/// Every user sees all areas: they belong to the organisation, and seeing an
/// area shows none of its flights.
pub async fn list(db: &PgPool, _actor: &Actor) -> ApiResult<Vec<Area>> {
    let mut conn = db.acquire().await?;
    Ok(db::list(&mut conn).await?)
}

/// An area as saved, and the areas it overlapped and was clipped against.
#[derive(Serialize)]
pub struct Saved {
    pub area: Area,
    pub clipped_by: Vec<String>,
}

/// Creates an area (`id` None) or changes one, clipped where it overlaps the
/// others, and recomputes every flight's figures per area.
pub async fn save(
    db: &PgPool,
    actor: &Actor,
    id: Option<Uuid>,
    name: &str,
    boundary: &Value,
) -> ApiResult<Saved> {
    policy::require_admin(actor)?;
    let name = error::name(name)?;
    let drawn = geometry::parse(boundary)
        .ok_or_else(|| ApiError::Invalid("Draw the area on the map.".to_owned()))?;

    let mut tx = db.begin().await?;
    db::lock_for_change(&mut tx).await?;
    let others: Vec<(String, _)> = db::list(&mut tx)
        .await?
        .into_iter()
        .filter(|area| Some(area.id) != id)
        .filter_map(|area| Some((area.name, geometry::parse(&area.boundary)?)))
        .collect();
    let shapes: Vec<_> = others.iter().map(|(_, shape)| shape.clone()).collect();
    let boundary = geometry::clip(&drawn, &shapes).ok_or_else(|| {
        ApiError::Invalid(
            "Everything you drew overlaps other areas. Draw at least part of it where no other \
             area is."
                .to_owned(),
        )
    })?;
    let clipped_by = geometry::overlapping(&drawn, &shapes)
        .into_iter()
        .map(|i| others[i].0.clone())
        .collect();
    let boundary = geometry::to_geojson(&boundary);

    let id = match id {
        Some(id) => {
            if !db::update(&mut tx, id, &name, &boundary)
                .await
                .map_err(name_taken)?
            {
                return Err(ApiError::NotFound);
            }
            id
        }
        None => db::insert(&mut tx, &name, &boundary)
            .await
            .map_err(name_taken)?,
    };
    db::recompute_all(&mut tx).await?;
    tx.commit().await?;
    Ok(Saved {
        area: Area { id, name, boundary },
        clipped_by,
    })
}

/// Deletes an area and recomputes every flight's figures per area.
pub async fn delete(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    policy::require_admin(actor)?;
    let mut tx = db.begin().await?;
    db::lock_for_change(&mut tx).await?;
    if !db::delete(&mut tx, id).await? {
        return Err(ApiError::NotFound);
    }
    db::recompute_all(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

fn name_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict("An area with this name already exists.".to_owned())
    } else {
        error.into()
    }
}
