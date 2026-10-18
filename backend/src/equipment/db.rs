//! The tables are separate, and a table can't be a query parameter, so every
//! query is written once per kind.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum Kind {
    Aircraft,
    RemoteController,
}

#[derive(Serialize)]
pub struct Equipment {
    pub id: Uuid,
    pub serial: String,
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub holder_user_id: Option<Uuid>,
    pub holder_name: Option<String>,
    pub last_flight_at: Option<DateTime<Utc>>,
}

/// What an admin sets on a drone or controller.
pub struct EquipmentFields<'a> {
    pub name: Option<&'a str>,
    pub team_id: Option<Uuid>,
    pub holder_user_id: Option<Uuid>,
}

/// All of one kind, by name or else serial.
pub async fn list(db: &PgPool, kind: Kind) -> sqlx::Result<Vec<Equipment>> {
    match kind {
        Kind::Aircraft => {
            sqlx::query_as!(
                Equipment,
                r#"select e.id, e.serial, e.name, e.team_id, t.name as "team_name?",
                          e.holder_user_id, p.name as "holder_name?",
                          (select max(takeoff_at) from flights f
                           where f.aircraft_serial = e.serial) as last_flight_at
                   from aircraft e
                   left join teams t on t.id = e.team_id
                   left join users p on p.id = e.holder_user_id
                   order by lower(coalesce(e.name, e.serial)), e.id"#,
            )
            .fetch_all(db)
            .await
        }
        Kind::RemoteController => {
            sqlx::query_as!(
                Equipment,
                r#"select e.id, e.serial, e.name, e.team_id, t.name as "team_name?",
                          e.holder_user_id, p.name as "holder_name?",
                          (select max(takeoff_at) from flights f
                           where f.rc_serial = e.serial) as last_flight_at
                   from remote_controllers e
                   left join teams t on t.id = e.team_id
                   left join users p on p.id = e.holder_user_id
                   order by lower(coalesce(e.name, e.serial)), e.id"#,
            )
            .fetch_all(db)
            .await
        }
    }
}

pub async fn get(db: &PgPool, kind: Kind, id: Uuid) -> sqlx::Result<Option<Equipment>> {
    match kind {
        Kind::Aircraft => {
            sqlx::query_as!(
                Equipment,
                r#"select e.id, e.serial, e.name, e.team_id, t.name as "team_name?",
                          e.holder_user_id, p.name as "holder_name?",
                          (select max(takeoff_at) from flights f
                           where f.aircraft_serial = e.serial) as last_flight_at
                   from aircraft e
                   left join teams t on t.id = e.team_id
                   left join users p on p.id = e.holder_user_id
                   where e.id = $1"#,
                id,
            )
            .fetch_optional(db)
            .await
        }
        Kind::RemoteController => {
            sqlx::query_as!(
                Equipment,
                r#"select e.id, e.serial, e.name, e.team_id, t.name as "team_name?",
                          e.holder_user_id, p.name as "holder_name?",
                          (select max(takeoff_at) from flights f
                           where f.rc_serial = e.serial) as last_flight_at
                   from remote_controllers e
                   left join teams t on t.id = e.team_id
                   left join users p on p.id = e.holder_user_id
                   where e.id = $1"#,
                id,
            )
            .fetch_optional(db)
            .await
        }
    }
}

/// False if there is no such item.
pub async fn update(
    db: &PgPool,
    kind: Kind,
    id: Uuid,
    fields: &EquipmentFields<'_>,
) -> sqlx::Result<bool> {
    let result = match kind {
        Kind::Aircraft => {
            sqlx::query!(
                "update aircraft set name = $2, team_id = $3, holder_user_id = $4
                 where id = $1",
                id,
                fields.name,
                fields.team_id,
                fields.holder_user_id,
            )
            .execute(db)
            .await?
        }
        Kind::RemoteController => {
            sqlx::query!(
                "update remote_controllers set name = $2, team_id = $3, holder_user_id = $4
                 where id = $1",
                id,
                fields.name,
                fields.team_id,
                fields.holder_user_id,
            )
            .execute(db)
            .await?
        }
    };
    Ok(result.rows_affected() == 1)
}

pub async fn user_is_active(db: &PgPool, user_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from users where id = $1 and archived_at is null) as "exists!""#,
        user_id,
    )
    .fetch_one(db)
    .await
}

pub async fn team_is_active(db: &PgPool, team_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from teams where id = $1 and archived_at is null) as "exists!""#,
        team_id,
    )
    .fetch_one(db)
    .await
}
