use sqlx::PgPool;

use super::db::{self, FlightSummary, Sort};
use crate::{auth::Actor, policy};

/// One page of the flights the actor may see, and how many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    sort: &Sort,
    limit: i64,
    offset: i64,
) -> sqlx::Result<(Vec<FlightSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let flights = db::list(db, &scope, sort, limit, offset).await?;
    let total = db::count(db, &scope).await?;
    Ok((flights, total))
}
