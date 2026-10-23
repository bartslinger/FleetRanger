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

/// The areas a drawing became, and the areas it overlapped and was clipped
/// against.
#[derive(Serialize)]
pub struct Saved {
    /// One area, or more where clipping split the drawing: separate areas
    /// with the same name (user's decision, 2026-10-21). The first one is the
    /// largest; a changed area keeps its id with it.
    pub areas: Vec<Area>,
    pub clipped_by: Vec<String>,
}

/// What a drawing is for: a new area with this name, or the area with this
/// id, which keeps its name.
pub enum Drawing<'a> {
    New(&'a str),
    Reshape(Uuid),
}

/// Creates an area, or changes the shape of one, clipped where it overlaps
/// the others, and recomputes every flight's figures per area.
pub async fn save(
    db: &PgPool,
    actor: &Actor,
    drawing: Drawing<'_>,
    boundary: &Value,
) -> ApiResult<Saved> {
    policy::require_admin(actor)?;
    let drawn = geometry::parse(boundary)
        .ok_or_else(|| ApiError::Invalid("Draw the area on the map.".to_owned()))?;

    let mut tx = db.begin().await?;
    db::lock_for_change(&mut tx).await?;
    let all = db::list(&mut tx).await?;
    let (id, name) = match drawing {
        Drawing::New(name) => (None, error::name(name)?),
        Drawing::Reshape(id) => {
            let area = all.iter().find(|area| area.id == id);
            (Some(id), area.ok_or(ApiError::NotFound)?.name.clone())
        }
    };
    let others: Vec<(String, _)> = all
        .into_iter()
        .filter(|area| Some(area.id) != id)
        .filter_map(|area| Some((area.name, geometry::parse(&area.boundary)?)))
        .collect();
    let shapes: Vec<_> = others.iter().map(|(_, shape)| shape.clone()).collect();
    let pieces = geometry::clip(&drawn, &shapes).ok_or_else(|| {
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

    let mut areas = Vec::new();
    for (index, piece) in pieces.into_iter().enumerate() {
        let boundary = geometry::to_geojson(&piece);
        let id = match id {
            Some(id) if index == 0 => {
                if !db::reshape(&mut tx, id, &boundary).await? {
                    return Err(ApiError::NotFound);
                }
                id
            }
            _ => db::insert(&mut tx, &name, &boundary).await?,
        };
        areas.push(Area {
            id,
            name: name.clone(),
            boundary,
        });
    }
    db::recompute_all(&mut tx).await?;
    tx.commit().await?;
    Ok(Saved { areas, clipped_by })
}

/// Renames an area; its shape and figures stay as they are.
pub async fn rename(db: &PgPool, actor: &Actor, id: Uuid, name: &str) -> ApiResult<()> {
    policy::require_admin(actor)?;
    let name = error::name(name)?;
    if !db::rename(db, id, &name).await? {
        return Err(ApiError::NotFound);
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{Grants, Role};
    use serde_json::json;

    fn admin() -> Actor {
        Actor {
            user_id: Uuid::nil(),
            role: Role::Admin,
            grants: Grants::from_stored([(None, "admin".to_owned())]).unwrap(),
            login_session_id: Uuid::nil(),
            ip: [127, 0, 0, 1].into(),
            email: String::new(),
            name: String::new(),
        }
    }

    fn rectangle(west: f64, east: f64, south: f64, north: f64) -> Value {
        json!({ "type": "Polygon", "coordinates": [[
            [west, south], [east, south], [east, north], [west, north], [west, south]
        ]] })
    }

    #[sqlx::test]
    async fn a_drawing_split_by_another_area_becomes_separate_areas(db: PgPool) {
        // A narrow strip from north to south, then an area drawn across it.
        save(
            &db,
            &admin(),
            Drawing::New("Test Road"),
            &rectangle(0.004, 0.006, 0.0, 0.01),
        )
        .await
        .ok()
        .unwrap();
        let saved = save(
            &db,
            &admin(),
            Drawing::New("Test Block"),
            &rectangle(0.0, 0.01, 0.002, 0.008),
        )
        .await
        .ok()
        .unwrap();

        assert_eq!(saved.areas.len(), 2);
        assert!(saved.areas.iter().all(|area| area.name == "Test Block"));
        assert!(
            saved
                .areas
                .iter()
                .all(|area| area.boundary["type"] == "Polygon")
        );
        assert_eq!(saved.clipped_by, ["Test Road"]);
        let mut conn = db.acquire().await.unwrap();
        assert_eq!(db::list(&mut conn).await.unwrap().len(), 3);
    }
}
