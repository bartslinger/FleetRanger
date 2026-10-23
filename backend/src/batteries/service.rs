use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Battery, BatteryFlight};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Permission},
};

/// A battery with its flights.
#[derive(serde::Serialize)]
pub struct BatteryDetail {
    #[serde(flatten)]
    pub battery: Battery,
    pub flight_list: Vec<BatteryFlight>,
}

pub async fn list(db: &PgPool, actor: &Actor) -> ApiResult<Vec<Battery>> {
    Ok(db::list(db, &policy::flight_scope(actor), None).await?)
}

/// One without a flight the actor sees is not found, so its existence isn't
/// revealed.
pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<BatteryDetail> {
    let scope = policy::flight_scope(actor);
    let battery = db::list(db, &scope, Some(id))
        .await?
        .pop()
        .ok_or(ApiError::NotFound)?;
    let flight_list = db::flights(db, &scope, id).await?;
    Ok(BatteryDetail {
        battery,
        flight_list,
    })
}

/// A battery has no group, so naming it needs `batteries.rename` on the
/// organisation.
pub async fn rename(
    db: &PgPool,
    actor: &Actor,
    id: Uuid,
    name: Option<&str>,
) -> ApiResult<BatteryDetail> {
    get(db, actor, id).await?;
    policy::require(actor, Permission::BatteriesRename, None)?;
    let name = name.map(str::trim).filter(|name| !name.is_empty());
    if !db::rename(db, id, name).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}
