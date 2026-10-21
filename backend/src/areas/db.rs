use serde::Serialize;
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::geometry::{self, Shape};

/// Flights are recomputed this many at a time, so their tracks aren't all in
/// memory at once.
const RECOMPUTE_BATCH: i64 = 200;

#[derive(Serialize)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    /// GeoJSON Polygon.
    pub boundary: Value,
}

/// All areas by name.
pub async fn list(db: &mut PgConnection) -> sqlx::Result<Vec<Area>> {
    sqlx::query_as!(
        Area,
        "select id, name, boundary from areas order by lower(name), id"
    )
    .fetch_all(db)
    .await
}

// Saving areas and saving flights take a lock on `areas`, so that a flight is
// never computed against areas that are about to change, and two areas being
// saved at once can't be clipped without seeing each other. Saving areas
// conflicts with itself and with saving flights; saving flights doesn't
// conflict with itself.

/// Locks the areas for changing them, until the transaction ends.
pub async fn lock_for_change(tx: &mut PgConnection) -> sqlx::Result<()> {
    sqlx::query!("lock table areas in share row exclusive mode")
        .execute(tx)
        .await?;
    Ok(())
}

pub async fn insert(tx: &mut PgConnection, name: &str, boundary: &Value) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "insert into areas (name, boundary) values ($1, $2) returning id",
        name,
        boundary,
    )
    .fetch_one(tx)
    .await
}

/// False if there is no such area.
pub async fn reshape(tx: &mut PgConnection, id: Uuid, boundary: &Value) -> sqlx::Result<bool> {
    let result = sqlx::query!("update areas set boundary = $2 where id = $1", id, boundary)
        .execute(tx)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// False if there is no such area.
pub async fn rename(db: &PgPool, id: Uuid, name: &str) -> sqlx::Result<bool> {
    let result = sqlx::query!("update areas set name = $2 where id = $1", id, name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// False if there is no such area. Its flight figures go with it.
pub async fn delete(tx: &mut PgConnection, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!("delete from areas where id = $1", id)
        .execute(tx)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// The areas, prepared for finding track points in them.
async fn shapes(tx: &mut PgConnection) -> sqlx::Result<(Vec<Uuid>, Vec<Shape>)> {
    let areas = list(tx).await?;
    let mut ids = Vec::new();
    let mut shapes = Vec::new();
    for area in areas {
        // Boundaries are checked when saved, so one that doesn't parse is skipped.
        if let Some(boundary) = geometry::parse(&area.boundary) {
            ids.push(area.id);
            shapes.push(Shape::new(boundary));
        }
    }
    Ok((ids, shapes))
}

struct TrackedFlight {
    id: Uuid,
    air_time_s: i32,
    distance_m: f64,
    track: Value,
}

/// Replaces the per-area figures of these flights.
async fn store(
    tx: &mut PgConnection,
    flights: &[TrackedFlight],
    ids: &[Uuid],
    shapes: &[Shape],
) -> sqlx::Result<()> {
    let (mut flight_ids, mut area_ids, mut times, mut distances) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for flight in flights {
        let track = geometry::track_points(&flight.track);
        for share in geometry::shares(&track, shapes, flight.air_time_s, flight.distance_m) {
            flight_ids.push(flight.id);
            area_ids.push(ids[share.area]);
            times.push(share.air_time_s);
            distances.push(share.distance_m);
        }
    }
    let all: Vec<Uuid> = flights.iter().map(|flight| flight.id).collect();
    sqlx::query!("delete from flight_areas where flight_id = any($1)", &all)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "insert into flight_areas (flight_id, area_id, air_time_s, distance_m)
         select * from unnest($1::uuid[], $2::uuid[], $3::int[], $4::float8[])",
        &flight_ids,
        &area_ids,
        &times,
        &distances,
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Computes a flight's figures per area, after it was saved in the same
/// transaction.
pub async fn compute_flight(tx: &mut PgConnection, flight_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!("lock table areas in share mode")
        .execute(&mut *tx)
        .await?;
    let (ids, shapes) = shapes(tx).await?;
    let flights = sqlx::query_as!(
        TrackedFlight,
        "select id, air_time_s, distance_m, track from flights where id = $1",
        flight_id,
    )
    .fetch_all(&mut *tx)
    .await?;
    store(tx, &flights, &ids, &shapes).await
}

/// Recomputes every flight's figures per area, archived flights included,
/// after the areas changed (user's decision, 2026-10-21). Needs
/// [`lock_for_change`].
pub async fn recompute_all(tx: &mut PgConnection) -> sqlx::Result<()> {
    let (ids, shapes) = shapes(tx).await?;
    let mut after = Uuid::nil();
    loop {
        let flights = sqlx::query_as!(
            TrackedFlight,
            "select id, air_time_s, distance_m, track from flights
             where id > $1 order by id limit $2",
            after,
            RECOMPUTE_BATCH,
        )
        .fetch_all(&mut *tx)
        .await?;
        let Some(last) = flights.last() else {
            return Ok(());
        };
        after = last.id;
        store(tx, &flights, &ids, &shapes).await?;
    }
}
