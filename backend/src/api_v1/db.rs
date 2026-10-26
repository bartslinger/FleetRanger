//! The public API's queries. What a user sees follows the flight queries'
//! visibility condition (see flights/db.rs and sessions/db.rs); keep them in
//! step. Pages are found by keyset: each list has a fixed order, and a page
//! starts after the last item of the one before.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{flights::db::Filter, policy::FlightScope};

/// Where a page of flights or sessions starts: after this time and id,
/// newest first.
#[derive(Clone, Copy)]
pub struct After {
    pub at: DateTime<Utc>,
    pub id: Uuid,
}

/// Which session a flight must be in, besides the shared filter.
#[derive(Clone, Copy, Default)]
pub struct InSession<'a> {
    /// One of these sessions, if given.
    pub ids: Option<&'a [Uuid]>,
    /// Or none at all.
    pub none: bool,
}

pub struct FlightRow {
    pub id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub distance_m: f64,
    pub max_height_m: Option<f64>,
    pub pilot_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub drone_id: Option<Uuid>,
    pub controller_id: Option<Uuid>,
    pub battery_ids: Vec<Uuid>,
    /// `[{area, air_time_s, distance_m}]`, most time first.
    pub areas: Value,
}

/// A flight's or session's time and distance in one area, as stored.
#[derive(Deserialize)]
pub struct AreaRow {
    pub area: Uuid,
    pub air_time_s: i64,
    pub distance_m: f64,
}

/// Flights in the scope that pass the filter (and are one of `ids`, if
/// given), newest first: a page after `after`, or all with `limit` None.
/// Archived flights aren't listed.
pub async fn flights(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    session: InSession<'_>,
    ids: Option<&[Uuid]>,
    after: Option<After>,
    limit: Option<i64>,
) -> sqlx::Result<Vec<FlightRow>> {
    sqlx::query_as!(
        FlightRow,
        r#"select f.id, f.takeoff_at, f.landed_at, f.air_time_s, f.distance_m,
                  (f.metrics->>'max_altitude_m')::float8 as max_height_m,
                  f.pilot_id, f.group_id, f.session_id,
                  (select a.id from aircraft a where a.serial = f.aircraft_serial) as drone_id,
                  (select r.id from remote_controllers r where r.serial = f.rc_serial)
                      as controller_id,
                  array(select fb.battery_id from flight_batteries fb
                        where fb.flight_id = f.id order by fb.slot) as "battery_ids!",
                  coalesce((select jsonb_agg(jsonb_build_object('area', fa.area_id,
                                                                'air_time_s', fa.air_time_s,
                                                                'distance_m', fa.distance_m)
                                             order by fa.air_time_s desc, fa.area_id)
                            from flight_areas fa where fa.flight_id = f.id),
                           '[]') as "areas!"
           from flights f
           where f.archived_at is null
             and ($1 or f.pilot_id = $2 or f.group_id = any($3))
             and ($4::text[] is null or f.aircraft_serial = any($4))
             and ($5::text[] is null or f.rc_serial = any($5))
             and ($6::timestamptz is null or f.takeoff_at >= $6)
             and ($7::timestamptz is null or f.takeoff_at < $7)
             and ($8::uuid[] is null or f.group_id = any($8) or ($9 and f.group_id is null))
             and ($10::uuid[] is null or f.pilot_id = any($10) or ($11 and f.pilot_id is null))
             and ($12::uuid[] is null or exists (select 1 from flight_areas fa
                                                 where fa.flight_id = f.id
                                                   and fa.area_id = any($12)))
             and (($13::uuid[] is null and not $14)
                  or f.session_id = any($13)
                  or ($14 and f.session_id is null))
             and ($15::timestamptz is null or (f.takeoff_at, f.id) < ($15, $16))
             and ($18::uuid[] is null or f.id = any($18))
           order by f.takeoff_at desc, f.id desc
           limit $17"#,
        scope.all,
        scope.user_id,
        &scope.group_ids,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        session.ids,
        session.none,
        after.map(|after| after.at),
        after.map(|after| after.id),
        limit,
        ids,
    )
    .fetch_all(db)
    .await
}

pub struct SessionRow {
    pub id: Uuid,
    pub group_id: Option<Uuid>,
    pub first_takeoff_at: DateTime<Utc>,
    pub last_landed_at: DateTime<Utc>,
    pub flight_count: i64,
    pub air_time_s: i64,
    pub distance_m: f64,
    pub pilot_ids: Vec<Uuid>,
    /// `[{area, air_time_s, distance_m}]` over its visible flights, most
    /// time first.
    pub areas: Value,
}

// As in the Sessions list: a session passes when one of its visible flights
// does, except that the group filter is on the session's own group; its
// figures are those of all its visible flights.

/// Sessions in the scope that pass the filter (or these ones, if given),
/// newest first by first takeoff: a page after `after`, or all with `limit`
/// None.
pub async fn sessions(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    ids: Option<&[Uuid]>,
    after: Option<After>,
    limit: Option<i64>,
) -> sqlx::Result<Vec<SessionRow>> {
    sqlx::query_as!(
        SessionRow,
        r#"select f.session_id as "id!", s.group_id,
                  min(f.takeoff_at) as "first_takeoff_at!",
                  max(f.landed_at) as "last_landed_at!",
                  count(*) as "flight_count!",
                  sum(f.air_time_s)::bigint as "air_time_s!",
                  sum(f.distance_m) as "distance_m!",
                  array_remove(array_agg(distinct f.pilot_id), null) as "pilot_ids!",
                  coalesce((select jsonb_agg(jsonb_build_object('area', t.area_id,
                                                                'air_time_s', t.air_time_s,
                                                                'distance_m', t.distance_m)
                                             order by t.air_time_s desc, t.area_id)
                            from (select fa.area_id, sum(fa.air_time_s) as air_time_s,
                                         sum(fa.distance_m) as distance_m
                                  from flight_areas fa
                                  join flights v on v.id = fa.flight_id
                                  where v.session_id = f.session_id
                                    and ($1 or v.pilot_id = $2 or v.group_id = any($3))
                                  group by fa.area_id) t),
                           '[]') as "areas!"
           from flights f
           join sessions s on s.id = f.session_id
           where ($1 or f.pilot_id = $2 or f.group_id = any($3))
             and ($4::uuid[] is null or f.session_id = any($4))
             and f.session_id in (
                 select g.session_id from flights g
                 where g.session_id is not null
                   and ($1 or g.pilot_id = $2 or g.group_id = any($3))
                   and ($5::text[] is null or g.aircraft_serial = any($5))
                   and ($6::text[] is null or g.rc_serial = any($6))
                   and ($7::timestamptz is null or g.takeoff_at >= $7)
                   and ($8::timestamptz is null or g.takeoff_at < $8)
                   and ($9::uuid[] is null
                        or g.session_id in (select id from sessions
                                            where group_id = any($9)
                                               or ($10 and group_id is null)))
                   and ($11::uuid[] is null or g.pilot_id = any($11)
                        or ($12 and g.pilot_id is null))
                   and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                       where fa.flight_id = g.id
                                                         and fa.area_id = any($13))))
           group by f.session_id, s.group_id
           having $14::timestamptz is null or (min(f.takeoff_at), f.session_id) < ($14, $15)
           order by min(f.takeoff_at) desc, f.session_id desc
           limit $16"#,
        scope.all,
        scope.user_id,
        &scope.group_ids,
        ids,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        after.map(|after| after.at),
        after.map(|after| after.id),
        limit,
    )
    .fetch_all(db)
    .await
}

pub struct CommentRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

/// The comments of these sessions, oldest first.
pub async fn comments(db: &PgPool, session_ids: &[Uuid]) -> sqlx::Result<Vec<CommentRow>> {
    sqlx::query_as!(
        CommentRow,
        "select id, session_id, author_id, body, created_at, edited_at
         from session_comments
         where session_id = any($1)
         order by created_at, id",
        session_ids
    )
    .fetch_all(db)
    .await
}

pub struct Named {
    pub id: Uuid,
    pub name: String,
}

pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub archived: bool,
}

pub struct DeviceRow {
    pub id: Uuid,
    pub name: Option<String>,
    pub serial: String,
    pub group_id: Option<Uuid>,
}

pub struct BatteryRow {
    pub id: Uuid,
    pub name: Option<String>,
    pub serial: String,
}

// What comes with a session or flight, looked up by id for `expand`. The
// session or flight was checked already: these show whatever it refers to.

pub async fn groups_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<Named>> {
    sqlx::query_as!(Named, "select id, name from groups where id = any($1)", ids)
        .fetch_all(db)
        .await
}

pub async fn areas_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<Named>> {
    sqlx::query_as!(Named, "select id, name from areas where id = any($1)", ids)
        .fetch_all(db)
        .await
}

pub async fn people_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<Person>> {
    sqlx::query_as!(
        Person,
        r#"select id, name, archived_at is not null as "archived!"
           from users where id = any($1)"#,
        ids
    )
    .fetch_all(db)
    .await
}

pub async fn drones_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<DeviceRow>> {
    sqlx::query_as!(
        DeviceRow,
        "select id, name, serial, group_id from aircraft where id = any($1)",
        ids
    )
    .fetch_all(db)
    .await
}

pub async fn controllers_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<DeviceRow>> {
    sqlx::query_as!(
        DeviceRow,
        "select id, name, serial, group_id from remote_controllers where id = any($1)",
        ids
    )
    .fetch_all(db)
    .await
}

pub async fn batteries_by_id(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<BatteryRow>> {
    sqlx::query_as!(
        BatteryRow,
        "select id, name, serial from batteries where id = any($1)",
        ids
    )
    .fetch_all(db)
    .await
}

/// The serials of these drones, for the flights' filter, which is by serial.
pub async fn drone_serials(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!("select serial from aircraft where id = any($1)", ids)
        .fetch_all(db)
        .await
}

pub async fn controller_serials(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        "select serial from remote_controllers where id = any($1)",
        ids
    )
    .fetch_all(db)
    .await
}

// The lists of groups and areas are in the order they were added, which ids
// follow; a page starts after the last id of the one before.

/// The groups where the user holds a role, or that a flight they see
/// belongs to; with a role on the organisation, all.
pub async fn groups(
    db: &PgPool,
    scope: &FlightScope,
    after: Option<Uuid>,
    limit: i64,
) -> sqlx::Result<Vec<Named>> {
    sqlx::query_as!(
        Named,
        "select g.id, g.name from groups g
         where ($1
                or exists (select 1 from group_members m
                           where m.group_id = g.id and m.user_id = $2)
                or exists (select 1 from flights f
                           where f.group_id = g.id and f.archived_at is null
                             and (f.pilot_id = $2 or f.group_id = any($3))))
           and ($4::uuid is null or g.id > $4)
         order by g.id
         limit $5",
        scope.all,
        scope.user_id,
        &scope.group_ids,
        after,
        limit,
    )
    .fetch_all(db)
    .await
}

pub struct ListedArea {
    pub id: Uuid,
    pub name: String,
    pub geometry: Option<Value>,
}

/// All the organisation's areas (user's decision, 2026-10-26: every user sees
/// them, as on the web app's maps), with their boundaries when asked.
pub async fn areas(
    db: &PgPool,
    geometry: bool,
    after: Option<Uuid>,
    limit: i64,
) -> sqlx::Result<Vec<ListedArea>> {
    sqlx::query_as!(
        ListedArea,
        "select id, name, case when $1 then boundary end as geometry from areas
         where $2::uuid is null or id > $2
         order by id
         limit $3",
        geometry,
        after,
        limit,
    )
    .fetch_all(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Adds a flight taking off at `takeoff` (UTC).
    async fn flight(db: &PgPool, takeoff: &str) -> Uuid {
        let raw_log: Uuid = sqlx::query_scalar(
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .fetch_one(db)
        .await
        .unwrap();
        let takeoff: DateTime<Utc> = takeoff.parse().unwrap();
        sqlx::query_scalar(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m, track)
             values ($1, $2, $2, 60, 100, '{}') returning id",
        )
        .bind(raw_log)
        .bind(takeoff)
        .fetch_one(db)
        .await
        .unwrap()
    }

    /// Pages follow on from each other without gaps or repeats, also through
    /// flights that took off at the same moment, newest first.
    #[sqlx::test]
    async fn pages_list_every_flight_once(db: PgPool) {
        let mut expected = Vec::new();
        for takeoff in [
            "2026-10-01T10:00:00Z",
            "2026-10-02T10:00:00Z",
            "2026-10-02T10:00:00Z",
            "2026-10-02T10:00:00Z",
            "2026-10-03T10:00:00Z",
        ] {
            expected.push(flight(&db, takeoff).await);
        }
        let admin = crate::testing::user(&db, "Admin", &["admin"]).await;
        let scope = crate::testing::scope(&db, admin).await;

        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = flights(
                &db,
                &scope,
                &Filter::default(),
                InSession::default(),
                None,
                after,
                Some(2),
            )
            .await
            .unwrap();
            let Some(last) = page.last() else { break };
            after = Some(After {
                at: last.takeoff_at,
                id: last.id,
            });
            seen.extend(page.iter().map(|flight| flight.id));
        }
        let mut sorted = seen.clone();
        sorted.sort();
        sorted.dedup();
        expected.sort();
        assert_eq!(sorted, expected);
        assert_eq!(seen.len(), expected.len());
    }

    /// An app sees what its user sees: a member of North gets North's
    /// flights only, and a session shared with South's flights lists and
    /// counts only North's.
    #[sqlx::test]
    async fn an_app_sees_only_its_users_flights(db: PgPool) {
        let north = crate::testing::group(&db, "North").await;
        let south = crate::testing::group(&db, "South").await;
        let member = crate::testing::user(&db, "Test Member", &[]).await;
        crate::testing::join(&db, member, north).await;
        let session: Uuid = sqlx::query_scalar("insert into sessions default values returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        let ours = flight(&db, "2026-10-01T10:00:00Z").await;
        let theirs = flight(&db, "2026-10-01T11:00:00Z").await;
        let elsewhere = flight(&db, "2026-10-02T10:00:00Z").await;
        for (id, group, in_session) in [
            (ours, north, true),
            (theirs, south, true),
            (elsewhere, south, false),
        ] {
            sqlx::query("update flights set group_id = $2, session_id = case when $3 then $4 end where id = $1")
                .bind(id)
                .bind(group)
                .bind(in_session)
                .bind(session)
                .execute(&db)
                .await
                .unwrap();
        }
        let scope = crate::testing::scope(&db, member).await;

        let seen = flights(
            &db,
            &scope,
            &Filter::default(),
            InSession::default(),
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(seen.iter().map(|f| f.id).collect::<Vec<_>>(), vec![ours]);
        let fetched = flights(
            &db,
            &scope,
            &Filter::default(),
            InSession::default(),
            Some(&[theirs]),
            None,
            None,
        )
        .await
        .unwrap();
        assert!(fetched.is_empty());

        let sessions = sessions(&db, &scope, &Filter::default(), None, None, None)
            .await
            .unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].flight_count, 1);
        assert_eq!(sessions[0].air_time_s, 60);
    }
}
