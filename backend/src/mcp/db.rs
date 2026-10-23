use chrono::{DateTime, NaiveDate, Utc};
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    flights::db::Filter,
    policy::{self, FlightScope, Permission},
};

/// The totals of one group of flights.
#[derive(Serialize)]
pub struct Totals {
    /// What the flights share: a team's, pilot's or area's id, a drone's or
    /// controller's serial, a day ("2026-10-12"), week ("2026-W42") or month
    /// ("2026-10"); none without grouping, or for flights without one.
    pub key: Option<String>,
    /// The team's, pilot's, drone's, controller's or area's name.
    pub name: Option<String>,
    pub flights: i64,
    /// Sessions with a flight in the group; one spanning two groups counts in both.
    pub sessions: i64,
    pub air_time_s: i64,
    /// In whole metres.
    pub distance_m: f64,
    pub first_takeoff_at: DateTime<Utc>,
    pub last_landed_at: DateTime<Utc>,
}

/// What totals can be grouped by.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GroupBy {
    None,
    Team,
    Pilot,
    Drone,
    Controller,
    Day,
    Week,
    Month,
    Area,
}

impl GroupBy {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Team => "team",
            Self::Pilot => "pilot",
            Self::Drone => "drone",
            Self::Controller => "controller",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Area => "area",
        }
    }
}

// The visibility condition and filters are the flight list's (flights::db),
// archived flights left out. Days, weeks and months are those of the
// organisation's time zone ($12).

/// The totals of the flights in the scope that pass the filter, per group:
/// named groups by name, periods in order.
pub async fn totals(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    group_by: GroupBy,
    time_zone: &str,
) -> sqlx::Result<Vec<Totals>> {
    sqlx::query_as!(
        Totals,
        r#"select key, name,
                  count(*) as "flights!",
                  count(distinct session_id) as "sessions!",
                  sum(air_time_s)::bigint as "air_time_s!",
                  round(sum(distance_m))::float8 as "distance_m!",
                  min(takeoff_at) as "first_takeoff_at!",
                  max(landed_at) as "last_landed_at!"
           from (
               select f.session_id, f.air_time_s, f.distance_m, f.takeoff_at, f.landed_at,
                      case $11
                          when 'team' then f.group_id::text
                          when 'pilot' then f.pilot_id::text
                          when 'drone' then f.aircraft_serial
                          when 'controller' then f.rc_serial
                          when 'day' then to_char(f.takeoff_at at time zone $12, 'YYYY-MM-DD')
                          when 'week' then to_char(f.takeoff_at at time zone $12, 'IYYY-"W"IW')
                          when 'month' then to_char(f.takeoff_at at time zone $12, 'YYYY-MM')
                      end as key,
                      case $11
                          when 'team' then (select g.name from groups g where g.id = f.group_id)
                          when 'pilot' then (select u.name from users u where u.id = f.pilot_id)
                          when 'drone' then (select a.name from aircraft a
                                             where a.serial = f.aircraft_serial)
                          when 'controller' then (select r.name from remote_controllers r
                                                  where r.serial = f.rc_serial)
                      end as name
               from flights f
               where f.archived_at is null
                 and ($1
                      or f.pilot_id = $2
                      or f.group_id = any($14))
                 and ($3::text[] is null or f.aircraft_serial = any($3))
                 and ($4::text[] is null or f.rc_serial = any($4))
                 and ($5::timestamptz is null or f.takeoff_at >= $5)
                 and ($6::timestamptz is null or f.takeoff_at < $6)
                 and ($7::uuid[] is null or f.group_id = any($7) or ($8 and f.group_id is null))
                 and ($9::uuid[] is null or f.pilot_id = any($9) or ($10 and f.pilot_id is null))
                 and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                     where fa.flight_id = f.id
                                                       and fa.area_id = any($13)))
           ) grouped
           group by key, name
           order by lower(name) nulls last, key nulls last"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.team_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        group_by.as_str(),
        time_zone,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// The time and distance in each area of the flights in the scope that pass
/// the filter, by area name: flights count in every area they have time in,
/// with only their time and distance there. Time outside all areas isn't
/// listed.
pub async fn area_totals(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<Totals>> {
    sqlx::query_as!(
        Totals,
        r#"select a.id::text as key, a.name as "name?",
                  count(*) as "flights!",
                  count(distinct f.session_id) as "sessions!",
                  sum(fa.air_time_s)::bigint as "air_time_s!",
                  round(sum(fa.distance_m))::float8 as "distance_m!",
                  min(f.takeoff_at) as "first_takeoff_at!",
                  max(f.landed_at) as "last_landed_at!"
           from flights f
           join flight_areas fa on fa.flight_id = f.id
           join areas a on a.id = fa.area_id
           where f.archived_at is null
             and ($1
                  or f.pilot_id = $2
                  or f.group_id = any($12))
             and ($3::text[] is null or f.aircraft_serial = any($3))
             and ($4::text[] is null or f.rc_serial = any($4))
             and ($5::timestamptz is null or f.takeoff_at >= $5)
             and ($6::timestamptz is null or f.takeoff_at < $6)
             and ($7::uuid[] is null or f.group_id = any($7) or ($8 and f.group_id is null))
             and ($9::uuid[] is null or f.pilot_id = any($9) or ($10 and f.pilot_id is null))
             and ($11::uuid[] is null or a.id = any($11))
           group by a.id, a.name
           order by lower(a.name), a.id"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.team_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// The instant a day starts in the time zone.
pub async fn start_of_day(
    db: &PgPool,
    day: NaiveDate,
    time_zone: &str,
) -> sqlx::Result<DateTime<Utc>> {
    sqlx::query_scalar!(
        r#"select ($1::date::timestamp at time zone $2) as "start!""#,
        day,
        time_zone,
    )
    .fetch_one(db)
    .await
}

/// A team the user flies for.
#[derive(Serialize)]
pub struct OwnTeam {
    pub id: Uuid,
    pub name: String,
}

/// The groups the user flies for, by name; none for someone who doesn't fly
/// in a group.
pub async fn own_teams(db: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<OwnTeam>> {
    sqlx::query_as!(
        OwnTeam,
        "select g.id, g.name from groups g
         where exists (select 1 from group_members m
                       where m.group_id = g.id and m.user_id = $1 and m.role = any($2))
         order by lower(g.name), g.id",
        user_id,
        &policy::roles_granting(Permission::FlightsFlyFor),
    )
    .fetch_all(db)
    .await
}

/// Today in the time zone; fails for a time zone Postgres doesn't know.
pub async fn today(db: &PgPool, time_zone: &str) -> sqlx::Result<NaiveDate> {
    sqlx::query_scalar!(
        r#"select (now() at time zone $1)::date as "today!""#,
        time_zone
    )
    .fetch_one(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    /// Adds a flight of this team taking off at `takeoff` (UTC), archived or not.
    async fn flight(db: &PgPool, team: Uuid, takeoff: &str, archived: bool) {
        let raw_log = insert_id(
            db,
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .await;
        let takeoff: DateTime<Utc> = takeoff.parse().unwrap();
        sqlx::query(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, group_id, archived_at)
             values ($1, $2, $2, 60, 100.4, '{}', $3, case when $4 then now() end)",
        )
        .bind(raw_log)
        .bind(takeoff)
        .bind(team)
        .bind(archived)
        .execute(db)
        .await
        .unwrap();
    }

    /// Months are the organisation's: a flight just before midnight UTC on 30
    /// September is in October in Johannesburg (UTC+2). Archived flights and
    /// flights of another team don't count for a member.
    #[sqlx::test]
    async fn totals_per_month_in_the_time_zone(db: PgPool) {
        let north = crate::testing::group(&db, "North").await;
        let south = crate::testing::group(&db, "South").await;
        let member = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        crate::testing::join(&db, member, north).await;
        flight(&db, north, "2026-09-30T12:00:00Z", false).await;
        flight(&db, north, "2026-09-30T23:30:00Z", false).await;
        flight(&db, north, "2026-09-30T12:00:00Z", true).await;
        flight(&db, south, "2026-09-30T12:00:00Z", false).await;

        let scope = crate::testing::scope(&db, member).await;
        let months = totals(
            &db,
            &scope,
            &Filter::default(),
            GroupBy::Month,
            "Africa/Johannesburg",
        )
        .await
        .unwrap();
        let rows: Vec<(Option<&str>, i64, f64)> = months
            .iter()
            .map(|row| (row.key.as_deref(), row.flights, row.distance_m))
            .collect();
        assert_eq!(
            rows,
            vec![(Some("2026-09"), 1, 100.0), (Some("2026-10"), 1, 100.0)]
        );
    }
}
