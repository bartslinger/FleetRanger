use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::Connection as _;
use sqlx::PgConnection;
use uuid::Uuid;

use super::SessionGap;
use crate::{
    flights::db::{Filter, GroupOption},
    policy::FlightScope,
};

// A session is visible when one of its flights is, and only the visible
// flights count. The visibility condition is the one in the flight queries
// (see flights/db.rs, which also lists a session's flights); keep them in step.

/// A session as listed: what its visible flights add up to.
#[derive(Serialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub first_takeoff_at: DateTime<Utc>,
    pub last_landed_at: DateTime<Utc>,
    pub flights: i64,
    pub air_time_s: i64,
    pub distance_m: f64,
    /// Names, in alphabetical order; empty if none.
    pub pilots: Vec<String>,
    /// The session's own group, which decides who changes it. Its flights
    /// may be in other groups.
    pub group_id: Uuid,
    pub group_name: String,
    /// Each visible flight's `takeoff_at`, `landed_at`, `air_time_s` and
    /// takeoff point (`from`, [lon, lat, alt]), in takeoff order: the web
    /// app's day strip, with day and night where the session was flown.
    /// Null, and left out, where it isn't wanted.
    #[serde(skip_serializing_if = "Value::is_null")]
    pub flight_times: Value,
}

/// A time span: sessions with a flight in it, e.g. the ones near flights to be
/// added to a session.
#[derive(Clone, Copy)]
pub struct Window {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

// The Sessions list's filters are the logbook's: a session passes when one of
// its visible flights does, except that the group filter is on the session's
// own group (user's decision, 2026-10-23). Its numbers stay those of all its
// visible flights.

/// One page of the sessions in the scope that pass the filter (and have a
/// flight in the window, if given), newest first.
pub async fn list(
    db: &mut PgConnection,
    scope: &FlightScope,
    filter: &Filter,
    window: Option<Window>,
    limit: i64,
    offset: i64,
    with_tracks: bool,
) -> sqlx::Result<Vec<SessionSummary>> {
    sqlx::query_as!(
        SessionSummary,
        r#"select f.session_id as "id!",
                min(f.takeoff_at) as "first_takeoff_at!",
                max(f.landed_at) as "last_landed_at!",
                count(*) as "flights!",
                sum(f.air_time_s)::bigint as "air_time_s!",
                sum(f.distance_m) as "distance_m!",
                array_remove(array_agg(distinct u.name), null) as "pilots!",
                s.group_id,
                gr.name as group_name,
                jsonb_agg(jsonb_build_object('takeoff_at', f.takeoff_at, 'landed_at', f.landed_at,
                                             'air_time_s', f.air_time_s,
                                             'from', f.track->'coordinates'->0,
                                             'track', case when $15 then f.track->'coordinates' end)
                          order by f.takeoff_at) as "flight_times!"
         from flights f
         join sessions s on s.id = f.session_id
         left join users u on u.id = f.pilot_id
         join groups gr on gr.id = s.group_id
         where f.session_id is not null
           and ($1
                or f.pilot_id = $2
                or f.group_id = any($16))
           and ($5::timestamptz is null
                or f.session_id in (select session_id from flights
                                    where takeoff_at < $6 and landed_at > $5))
           and f.session_id in (
               select g.session_id from flights g
               where g.session_id is not null
                 and ($1
                      or g.pilot_id = $2
                      or g.group_id = any($16))
                 and ($7::text[] is null or g.aircraft_serial = any($7))
                 and ($8::text[] is null or g.rc_serial = any($8))
                 and ($11::uuid[] is null
                      or g.session_id in (select id from sessions
                                          where group_id = any($11)))
                 and ($12::uuid[] is null or g.pilot_id = any($12)
                      or ($13 and g.pilot_id is null))
                 and ($14::uuid[] is null or exists (select 1 from flight_areas fa
                                                     where fa.flight_id = g.id
                                                       and fa.area_id = any($14))))
         group by f.session_id, s.group_id, gr.name
         having ($9::timestamptz is null or min(f.takeoff_at) >= $9)
            and ($10::timestamptz is null or min(f.takeoff_at) < $10)
         order by min(f.takeoff_at) desc, f.session_id
         limit $3 offset $4"#,
        scope.all,
        scope.user_id,
        limit,
        offset,
        window.map(|w| w.from),
        window.map(|w| w.to),
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        with_tracks,
        &scope.group_ids,
    )
    .fetch_all(&mut *db)
    .await
}

/// The number of sessions `list` lists in all.
pub async fn count(
    db: &mut PgConnection,
    scope: &FlightScope,
    filter: &Filter,
    window: Option<Window>,
) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!" from (
           select session_id
           from flights
           where ($1
                  or pilot_id = $2
                  or group_id = any($13))
             and ($3::timestamptz is null
                  or session_id in (select session_id from flights
                                    where takeoff_at < $4 and landed_at > $3))
             and session_id in (
                 select g.session_id from flights g
                 where g.session_id is not null
                   and ($1
                        or g.pilot_id = $2
                        or g.group_id = any($13))
                   and ($5::text[] is null or g.aircraft_serial = any($5))
                   and ($6::text[] is null or g.rc_serial = any($6))
                   and ($9::uuid[] is null
                        or g.session_id in (select id from sessions
                                            where group_id = any($9)))
                   and ($10::uuid[] is null or g.pilot_id = any($10)
                        or ($11 and g.pilot_id is null))
                   and ($12::uuid[] is null or exists (select 1 from flight_areas fa
                                                       where fa.flight_id = g.id
                                                         and fa.area_id = any($12))))
           group by session_id
           having ($7::timestamptz is null or min(takeoff_at) >= $7)
              and ($8::timestamptz is null or min(takeoff_at) < $8)
        ) listed"#,
        scope.all,
        scope.user_id,
        window.map(|w| w.from),
        window.map(|w| w.to),
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_one(&mut *db)
    .await
}

/// The sessions before (newer) and after (older) one in a list, with their
/// first takeoff, so stepping can go on into another week or month.
#[derive(Serialize)]
pub struct Neighbours {
    pub previous: Option<Uuid>,
    pub previous_start: Option<DateTime<Utc>>,
    pub next: Option<Uuid>,
    pub next_start: Option<DateTime<Utc>>,
}

/// Where a session is in the scope's list with this filter, newest first as
/// `list` orders it: the sessions before and after it. None if the session
/// isn't in that list.
pub async fn neighbours(
    db: &mut PgConnection,
    scope: &FlightScope,
    filter: &Filter,
    id: Uuid,
) -> sqlx::Result<Option<Neighbours>> {
    sqlx::query_as!(
        Neighbours,
        r#"with listed as (
               select f.session_id as id, min(f.takeoff_at) as start
               from flights f
               where f.session_id is not null
                 and ($1
                      or f.pilot_id = $2
                      or f.group_id = any($12))
                 and f.session_id in (
                     select g.session_id from flights g
                     where g.session_id is not null
                       and ($1
                            or g.pilot_id = $2
                            or g.group_id = any($12))
                       and ($4::text[] is null or g.aircraft_serial = any($4))
                       and ($5::text[] is null or g.rc_serial = any($5))
                       and ($8::uuid[] is null
                            or g.session_id in (select id from sessions
                                                where group_id = any($8)))
                       and ($9::uuid[] is null or g.pilot_id = any($9)
                            or ($10 and g.pilot_id is null))
                       and ($11::uuid[] is null or exists (select 1 from flight_areas fa
                                                           where fa.flight_id = g.id
                                                             and fa.area_id = any($11))))
               group by f.session_id
               having ($6::timestamptz is null or min(f.takeoff_at) >= $6)
                  and ($7::timestamptz is null or min(f.takeoff_at) < $7)
           ),
           placed as (
               select id,
                      lag(id) over listing as previous,
                      lag(start) over listing as previous_start,
                      lead(id) over listing as next,
                      lead(start) over listing as next_start
               from listed
               window listing as (order by start desc, id)
           )
           select previous, previous_start, next, next_start
           from placed
           where id = $3"#,
        scope.all,
        scope.user_id,
        id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_optional(&mut *db)
    .await
}

/// When the nearest sessions outside a period start: the last before it and
/// the first after it, so the cards' arrows skip empty weeks and months.
#[derive(Serialize, Debug, PartialEq)]
pub struct Around {
    pub before: Option<DateTime<Utc>>,
    pub after: Option<DateTime<Utc>>,
}

/// The nearest sessions in the scope's list with this filter that start
/// before `filter.from` and at or after `filter.to`, as `list` places
/// sessions in a period: by their first takeoff. Without `from` nothing is
/// before, without `to` nothing after.
pub async fn around(
    db: &mut PgConnection,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Around> {
    sqlx::query_as!(
        Around,
        r#"with listed as (
               select min(f.takeoff_at) as start
               from flights f
               where f.session_id is not null
                 and ($1
                      or f.pilot_id = $2
                      or f.group_id = any($11))
                 and f.session_id in (
                     select g.session_id from flights g
                     where g.session_id is not null
                       and ($1
                            or g.pilot_id = $2
                            or g.group_id = any($11))
                       and ($3::text[] is null or g.aircraft_serial = any($3))
                       and ($4::text[] is null or g.rc_serial = any($4))
                       and ($7::uuid[] is null
                            or g.session_id in (select id from sessions
                                                where group_id = any($7)))
                       and ($8::uuid[] is null or g.pilot_id = any($8)
                            or ($9 and g.pilot_id is null))
                       and ($10::uuid[] is null or exists (select 1 from flight_areas fa
                                                           where fa.flight_id = g.id
                                                             and fa.area_id = any($10))))
               group by f.session_id
           )
           select max(start) filter (where start < $5) as before,
                  min(start) filter (where start >= $6) as after
           from listed"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_one(&mut *db)
    .await
}

/// Any constant, so that only one change to an organisation's sessions
/// (grouping, splitting, merging) runs at a time; other organisations'
/// don't wait for it.
const SESSIONS_LOCK: i32 = 0x5e55_1015;

/// A transaction holding the sessions lock of the connection's organisation.
pub(crate) async fn locked(
    db: &mut PgConnection,
) -> sqlx::Result<sqlx::Transaction<'_, sqlx::Postgres>> {
    let mut tx = db.begin().await?;
    lock(&mut tx).await?;
    Ok(tx)
}

/// Takes the sessions lock of the connection's organisation until the end of
/// the transaction it is in.
pub(crate) async fn lock(db: &mut PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        "select pg_advisory_xact_lock($1, hashtext(coalesce(current_setting('app.organisation_id', true), '')))",
        SESSIONS_LOCK
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

/// The group of each of these sessions, or None if one of them doesn't
/// exist.
pub async fn groups_of(db: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<Option<Vec<Uuid>>> {
    let groups = sqlx::query_scalar!("select group_id from sessions where id = any($1)", ids)
        .fetch_all(&mut *db)
        .await?;
    Ok((groups.len() == ids.len()).then_some(groups))
}

/// A session's own group, by id and name.
#[derive(Serialize)]
pub struct OwnGroup {
    pub group_id: Uuid,
    pub group_name: String,
}

/// The group of a session, or None if it doesn't exist.
pub async fn own_group(db: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<OwnGroup>> {
    sqlx::query_as!(
        OwnGroup,
        r#"select s.group_id, g.name as group_name
           from sessions s join groups g on g.id = s.group_id
           where s.id = $1"#,
        id
    )
    .fetch_optional(&mut *db)
    .await
}

/// The sessions' groups to offer in the Sessions list's filter, as
/// `flights::db::groups_in` offers the flights' groups: those of sessions
/// with a visible flight passing the other filters, and the chosen ones.
pub async fn groups_in(
    db: &mut PgConnection,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<GroupOption>> {
    sqlx::query_as!(
        GroupOption,
        r#"select s.group_id as "id?", g.name as "name?"
           from sessions s
           left join groups g on g.id = s.group_id
           where s.group_id = any(coalesce($3::uuid[], '{}'))
              or exists (
                  select 1 from flights f
                  where f.session_id = s.id
                    and ($1 or f.pilot_id = $2 or f.group_id = any($11))
                    and f.archived_at is null
                    and ($4::text[] is null or f.aircraft_serial = any($4))
                    and ($5::text[] is null or f.rc_serial = any($5))
                    and ($8::uuid[] is null or f.pilot_id = any($8)
                         or ($9 and f.pilot_id is null))
                    and ($10::uuid[] is null or exists (select 1 from flight_areas fa
                                                        where fa.flight_id = f.id
                                                          and fa.area_id = any($10))))
                 -- In the period of its first visible takeoff, as listed.
                 and ($6::timestamptz is null and $7::timestamptz is null
                      or (select min(h.takeoff_at) >= coalesce($6, '-infinity')
                                 and min(h.takeoff_at) < coalesce($7, 'infinity')
                          from flights h
                          where h.session_id = s.id
                            and ($1 or h.pilot_id = $2 or h.group_id = any($11))))
           group by s.group_id, g.name
           order by lower(g.name) nulls last"#,
        scope.all,
        scope.user_id,
        filter.group_ids.as_deref(),
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(&mut *db)
    .await
}

/// The groups of the sessions these flights are in, once each; flights
/// without a session add none.
pub async fn groups_left(db: &mut PgConnection, flight_ids: &[Uuid]) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar!(
        "select s.group_id from sessions s
         where s.id in (select session_id from flights where id = any($1))",
        flight_ids
    )
    .fetch_all(&mut *db)
    .await
}

/// The group of every flight in these sessions, or None if one of them
/// doesn't exist. The visibility check is the policy's job, not this query's.
pub async fn flight_groups(db: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<Option<Vec<Uuid>>> {
    let flights = sqlx::query!(
        r#"select session_id as "session_id!", group_id from flights where session_id = any($1)"#,
        ids
    )
    .fetch_all(&mut *db)
    .await?;
    let found: HashSet<Uuid> = flights.iter().map(|flight| flight.session_id).collect();
    Ok((found.len() == ids.len())
        .then(|| flights.into_iter().map(|flight| flight.group_id).collect()))
}

/// The one session all these flights are in, or None if one of them doesn't
/// exist, isn't in a session, or they are in different sessions.
pub async fn session_of(db: &mut PgConnection, flight_ids: &[Uuid]) -> sqlx::Result<Option<Uuid>> {
    let sessions = sqlx::query_scalar!(
        "select session_id from flights where id = any($1)",
        flight_ids
    )
    .fetch_all(&mut *db)
    .await?;
    let first = sessions.first().copied().flatten();
    let same = sessions.len() == flight_ids.len() && sessions.iter().all(|s| *s == first);
    Ok(first.filter(|_| same))
}

/// Records that a person changed these sessions: automatic grouping leaves
/// them as they are from now on (merges them with no other).
async fn changed(tx: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<()> {
    sqlx::query!(
        "update sessions set changed_at = now() where id = any($1)",
        ids
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// A person puts all flights of these sessions in the one that starts first,
/// deletes the others, and returns the remaining session.
pub async fn merge(db: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<Uuid> {
    let mut tx = locked(db).await?;
    let target = merge_into_first(&mut tx, ids).await?;
    changed(&mut tx, &[target]).await?;
    tx.commit().await?;
    Ok(target)
}

/// Merges, as `merge` does, in the caller's transaction.
async fn merge_into_first(tx: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<Uuid> {
    let target = sqlx::query_scalar!(
        r#"select session_id as "session_id!" from flights where session_id = any($1)
           order by takeoff_at, id limit 1"#,
        ids
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "update flights set session_id = $2 where session_id = any($1)",
        ids,
        target
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "update session_comments set session_id = $2 where session_id = any($1)",
        ids,
        target
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "delete from sessions where id = any($1) and id <> $2",
        ids,
        target
    )
    .execute(&mut *tx)
    .await?;
    Ok(target)
}

// Moving flights out of sessions: a session left without flights is deleted,
// unless it has comments; then its last flight can't leave, and the change
// returns None without changing anything (user's decision, 2026-10-25).

/// Moves these flights into a new session of this group, and returns it.
/// Deletes a session left without flights.
pub async fn split(
    db: &mut PgConnection,
    flight_ids: &[Uuid],
    group_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    let mut tx = locked(db).await?;
    let left = sessions_of(&mut tx, flight_ids).await?;
    let session = sqlx::query_scalar!(
        "insert into sessions (group_id) values ($1) returning id",
        group_id
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "update flights set session_id = $2 where id = any($1)",
        flight_ids,
        session
    )
    .execute(&mut *tx)
    .await?;
    if stranded(&mut tx, &left).await? {
        return Ok(None);
    }
    changed(&mut tx, &[&left[..], &[session]].concat()).await?;
    delete_empty(&mut tx).await?;
    tx.commit().await?;
    Ok(Some(session))
}

/// Moves these flights into the session. Deletes a session left without
/// flights.
pub async fn add(
    db: &mut PgConnection,
    session: Uuid,
    flight_ids: &[Uuid],
) -> sqlx::Result<Option<()>> {
    let mut tx = locked(db).await?;
    let left = sessions_of(&mut tx, flight_ids).await?;
    sqlx::query!(
        "update flights set session_id = $2 where id = any($1)",
        flight_ids,
        session
    )
    .execute(&mut *tx)
    .await?;
    if stranded(&mut tx, &left).await? {
        return Ok(None);
    }
    changed(&mut tx, &[&left[..], &[session]].concat()).await?;
    delete_empty(&mut tx).await?;
    tx.commit().await?;
    Ok(Some(()))
}

/// Takes these flights out of their sessions, and returns how many were in
/// one. Deletes a session left without flights.
pub async fn remove(db: &mut PgConnection, flight_ids: &[Uuid]) -> sqlx::Result<Option<u64>> {
    let mut tx = locked(db).await?;
    let left = sessions_of(&mut tx, flight_ids).await?;
    let removed = sqlx::query!(
        "update flights set session_id = null where id = any($1) and session_id is not null",
        flight_ids
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if stranded(&mut tx, &left).await? {
        return Ok(None);
    }
    changed(&mut tx, &left).await?;
    delete_empty(&mut tx).await?;
    tx.commit().await?;
    Ok(Some(removed))
}

/// The sessions these flights are in.
async fn sessions_of(tx: &mut sqlx::PgConnection, flight_ids: &[Uuid]) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar!(
        r#"select distinct session_id as "session_id!" from flights
           where id = any($1) and session_id is not null"#,
        flight_ids
    )
    .fetch_all(tx)
    .await
}

/// Whether one of these sessions has comments but no flights any more.
async fn stranded(tx: &mut sqlx::PgConnection, ids: &[Uuid]) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from sessions s
                         where s.id = any($1)
                           and not exists (select 1 from flights f where f.session_id = s.id)
                           and exists (select 1 from session_comments c
                                       where c.session_id = s.id)) as "stranded!""#,
        ids
    )
    .fetch_one(tx)
    .await
}

/// Deletes a session and its comments; its flights stay, without a session.
/// Returns false if there is no such session.
pub async fn delete(db: &mut PgConnection, id: Uuid) -> sqlx::Result<bool> {
    let mut tx = locked(db).await?;
    sqlx::query!(
        "update flights set session_id = null where session_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("delete from session_comments where session_id = $1", id)
        .execute(&mut *tx)
        .await?;
    let found = sqlx::query!("delete from sessions where id = $1", id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
        == 1;
    tx.commit().await?;
    Ok(found)
}

/// Moves a session to a group, and with `with_flights` its flights as well. Returns false if there is no such
/// session.
pub async fn set_group(
    db: &mut PgConnection,
    id: Uuid,
    group_id: Uuid,
    with_flights: bool,
) -> sqlx::Result<bool> {
    let mut tx = locked(db).await?;
    let found = sqlx::query!(
        "update sessions set group_id = $2, changed_at = now() where id = $1",
        id,
        group_id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if found && with_flights {
        sqlx::query!(
            "update flights set group_id = $2 where session_id = $1",
            id,
            group_id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(found)
}

/// Deletes the sessions without flights, except those with comments.
async fn delete_empty(tx: &mut sqlx::PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        "delete from sessions s
         where not exists (select 1 from flights f where f.session_id = s.id)
           and not exists (select 1 from session_comments c where c.session_id = s.id)"
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Places the flight of a newly processed log, the one case where a flight
/// joins a session by itself, so uploaded and imported flights show up as
/// sessions (user's decision, 2026-11-18). It joins the nearest session of
/// the flight's group holding a flight less than the gap apart of the same
/// pilot, or, without a pilot (logs from before anyone chose one on the
/// controller, and imports), from the same controller, however late it comes
/// in (user's decision, 2026-11-18: an upload cut off may finish days later).
/// A pilot's flight also joins its controller's flights without a pilot, in a
/// session no other pilot flew in. With no such session, it starts a new one
/// of the flight's group. Sessions it bridges merge into one,
/// if nobody changed them (a comment is a change), so a backlog comes out the same
/// in any order; a person's changes are never undone.
pub async fn place_upload(
    db: &mut PgConnection,
    raw_log_id: Uuid,
    gap: SessionGap,
) -> sqlx::Result<()> {
    let mut tx = locked(db).await?;
    let Some(flight) = sqlx::query!(
        r#"select f.id, f.pilot_id, f.rc_serial, f.group_id, f.takeoff_at, f.landed_at
           from flights f
           where f.raw_log_id = $1 and f.session_id is null and f.archived_at is null
             and (f.pilot_id is not null or f.rc_serial is not null)"#,
        raw_log_id
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return tx.commit().await;
    };

    // The sessions it could join, nearest first: those of its group holding a
    // flight less than the gap away (the break, negative when they overlap)
    // of its pilot, or without a pilot from its controller; a flight without
    // a pilot goes by its controller alone. Never one with another pilot:
    // different pilots aren't grouped by themselves.
    let candidates = sqlx::query!(
        r#"select f.session_id as "session_id!",
                  (select array_agg(distinct o.pilot_id) filter (where o.pilot_id is not null)
                   from flights o where o.session_id = f.session_id) as pilots,
                  bool_and(s.changed_at is null) as "untouched!"
           from flights f
           join sessions s on s.id = f.session_id
           where s.group_id = $5
             and case when $1::uuid is null then f.rc_serial = $6
                      else f.pilot_id = $1
                           or (f.pilot_id is null and f.rc_serial = $6
                               and not exists (select 1 from flights o
                                               where o.session_id = f.session_id
                                                 and o.pilot_id <> $1)) end
             and f.takeoff_at < $3::timestamptz + make_interval(mins => $4)
             and f.landed_at > $2::timestamptz - make_interval(mins => $4)
           group by f.session_id
           order by min(greatest(f.takeoff_at - $3::timestamptz, $2::timestamptz - f.landed_at)),
                    f.session_id"#,
        flight.pilot_id,
        flight.takeoff_at,
        flight.landed_at,
        gap.0,
        flight.group_id,
        flight.rc_serial,
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut candidates = candidates.into_iter();
    let session = match candidates.next() {
        Some(nearest) => {
            // Sessions it bridges become one, so a backlog comes out the same
            // in any order; only sessions nobody changed
            // (user's decision, 2026-11-18), and never across pilots.
            let mut pilots: Vec<Uuid> = nearest.pilots.unwrap_or_default();
            pilots.extend(flight.pilot_id);
            pilots.sort_unstable();
            pilots.dedup();
            let mut merged = vec![nearest.session_id];
            if nearest.untouched {
                for other in candidates.filter(|other| other.untouched) {
                    let mut together = pilots.clone();
                    together.extend(other.pilots.unwrap_or_default());
                    together.sort_unstable();
                    together.dedup();
                    if together.len() <= 1 {
                        merged.push(other.session_id);
                        pilots = together;
                    }
                }
            }
            if merged.len() > 1 {
                merge_into_first(&mut tx, &merged).await?
            } else {
                nearest.session_id
            }
        }
        None => {
            sqlx::query_scalar!(
                "insert into sessions (group_id) values ($1) returning id",
                flight.group_id
            )
            .fetch_one(&mut *tx)
            .await?
        }
    };
    sqlx::query!(
        "update flights set session_id = $2 where id = $1",
        flight.id,
        session,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_id(db: &mut PgConnection, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(&mut *db).await.unwrap()
    }

    /// Adds a flight of this pilot, from and to times on the same day.
    async fn flight(db: &mut PgConnection, pilot: Uuid, takeoff: &str, landed: &str) -> Uuid {
        let at =
            |time: &str| -> DateTime<Utc> { format!("2026-10-12T{time}:00Z").parse().unwrap() };
        let raw_log = insert_id(
            db,
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .await;
        sqlx::query_scalar(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, pilot_id, group_id)
             select $1, $2, $3, 0, 0, '{}', $4, id from groups where name = 'General'
             returning id",
        )
        .bind(raw_log)
        .bind(at(takeoff))
        .bind(at(landed))
        .bind(pilot)
        .fetch_one(&mut *db)
        .await
        .unwrap()
    }

    /// Adds a flight of this pilot as a controller uploads it (its log carries
    /// the pilot), received on 12 Oct at `received` or on a later day's
    /// `"+1 12:00"`, and places it as the processing loop does.
    async fn upload(
        db: &mut PgConnection,
        pilot: Uuid,
        takeoff: &str,
        landed: &str,
        received: &str,
    ) -> Uuid {
        upload_for(db, pilot, None, takeoff, landed, received).await
    }

    /// As `upload`, with the group chosen on the controller (None: General).
    async fn upload_for(
        db: &mut PgConnection,
        pilot: Uuid,
        group: Option<Uuid>,
        takeoff: &str,
        landed: &str,
        received: &str,
    ) -> Uuid {
        let flight = flight(db, pilot, takeoff, landed).await;
        sqlx::query("update flights set group_id = coalesce($2, group_id) where id = $1")
            .bind(flight)
            .bind(group)
            .execute(&mut *db)
            .await
            .unwrap();
        let received: DateTime<Utc> = match received.strip_prefix("+1 ") {
            Some(time) => format!("2026-10-13T{time}:00Z"),
            None => format!("2026-10-12T{received}:00Z"),
        }
        .parse()
        .unwrap();
        let raw_log: Uuid = sqlx::query_scalar(
            "update raw_logs set pilot_id = $2, received_at = $3
             where id = (select raw_log_id from flights where id = $1)
             returning id",
        )
        .bind(flight)
        .bind(pilot)
        .bind(received)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        place_upload(db, raw_log, SessionGap(120)).await.unwrap();
        flight
    }

    async fn session_of(db: &mut PgConnection, flight: Uuid) -> Option<Uuid> {
        sqlx::query_scalar("select session_id from flights where id = $1")
            .bind(flight)
            .fetch_one(&mut *db)
            .await
            .unwrap()
    }

    /// Uploaded flights of a pilot less than the gap apart share a session,
    /// also when processed out of order; a longer break or another pilot
    /// starts a new one.
    #[sqlx::test]
    async fn uploads_group_by_pilot_and_gap(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let b = insert_id(
            &mut db,
            "insert into users (name) values ('B') returning id",
        )
        .await;

        let first = upload(&mut db, a, "18:12", "18:40", "21:30").await;
        let last = upload(&mut db, a, "20:30", "21:21", "21:30").await;
        let other_pilot = upload(&mut db, b, "18:20", "18:50", "21:30").await;
        assert_eq!(
            session_of(&mut db, first).await,
            session_of(&mut db, last).await
        );
        assert_ne!(
            session_of(&mut db, first).await,
            session_of(&mut db, other_pilot).await
        );

        // Processed later, in between: it joins, and doesn't move the others.
        let middle = upload(&mut db, a, "19:30", "19:50", "21:31").await;
        assert_eq!(
            session_of(&mut db, middle).await,
            session_of(&mut db, first).await
        );
        // More than two hours after the last landing.
        let later = upload(&mut db, a, "23:22", "23:40", "23:50").await;
        assert!(session_of(&mut db, later).await.is_some());
        assert_ne!(
            session_of(&mut db, later).await,
            session_of(&mut db, first).await
        );
    }

    /// A flight that comes in days after its session's flights (an upload cut
    /// off and finished later) still joins its outing.
    #[sqlx::test]
    async fn late_uploads_join_their_outing(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let synced = upload(&mut db, a, "18:12", "18:40", "21:30").await;
        assert!(session_of(&mut db, synced).await.is_some());

        let late = upload(&mut db, a, "19:00", "19:20", "+1 12:00").await;
        assert_eq!(
            session_of(&mut db, late).await,
            session_of(&mut db, synced).await
        );
    }

    /// Flights without a pilot (from before anyone chose one on the
    /// controller, or imported) group by their controller: a controller's
    /// flights less than the gap apart share a session, another controller's
    /// don't, and a flight without either stays out.
    #[sqlx::test]
    async fn flights_without_a_pilot_group_by_controller(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let mut place = async |takeoff, landed, controller: Option<&str>| {
            let flight = flight(&mut db, a, takeoff, landed).await;
            let raw_log: Uuid = sqlx::query_scalar(
                "update flights set pilot_id = null, rc_serial = $2 where id = $1
                 returning raw_log_id",
            )
            .bind(flight)
            .bind(controller)
            .fetch_one(&mut *db)
            .await
            .unwrap();
            place_upload(&mut db, raw_log, SessionGap(120))
                .await
                .unwrap();
            session_of(&mut db, flight).await
        };
        let first = place("08:00", "08:20", Some("Test RC1")).await;
        let second = place("09:00", "09:20", Some("Test RC1")).await;
        let other_controller = place("08:30", "08:50", Some("Test RC2")).await;
        let neither = place("08:40", "08:50", None).await;
        assert!(first.is_some());
        assert_eq!(first, second);
        assert!(other_controller.is_some());
        assert_ne!(first, other_controller);
        assert_eq!(neither, None);
    }

    /// Adds a flight from controller "Test RC1", with or without a pilot,
    /// received now, and places it as the processing loop does.
    async fn from_controller(
        db: &mut PgConnection,
        pilot: Option<Uuid>,
        takeoff: &str,
        landed: &str,
    ) -> Uuid {
        let any = insert_id(db, "insert into users (name) values ('Test') returning id").await;
        let flight = flight(db, any, takeoff, landed).await;
        let raw_log: Uuid = sqlx::query_scalar(
            "update flights set pilot_id = $2, rc_serial = 'Test RC1' where id = $1
             returning raw_log_id",
        )
        .bind(flight)
        .bind(pilot)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        place_upload(db, raw_log, SessionGap(120)).await.unwrap();
        flight
    }

    /// An outing's flights make one session in any order: a
    /// flight in between joins the sessions on both sides, and flights from
    /// before and after a pilot was chosen on the controller come together.
    /// Different pilots stay apart.
    #[sqlx::test]
    async fn one_outing_is_one_session_in_any_order(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let b = insert_id(
            &mut db,
            "insert into users (name) values ('B') returning id",
        )
        .await;

        // Four hours apart, then the flight in between.
        let early = from_controller(&mut db, None, "06:00", "06:20").await;
        let late = from_controller(&mut db, None, "10:00", "10:20").await;
        assert_ne!(
            session_of(&mut db, early).await,
            session_of(&mut db, late).await
        );
        let middle = from_controller(&mut db, None, "08:00", "08:20").await;
        let outing = session_of(&mut db, middle).await;
        assert!(outing.is_some());
        assert_eq!(session_of(&mut db, early).await, outing);
        assert_eq!(session_of(&mut db, late).await, outing);

        // A pilot chosen after those, with the pilot's flight first.
        let piloted = from_controller(&mut db, Some(a), "11:00", "11:20").await;
        assert_eq!(session_of(&mut db, piloted).await, outing);

        // Another pilot's flight next to A's doesn't join, nor does a flight
        // without a pilot bridge the two.
        let other = from_controller(&mut db, Some(b), "11:40", "11:50").await;
        assert_ne!(session_of(&mut db, other).await, outing);
        let between = from_controller(&mut db, None, "12:20", "12:30").await;
        assert_ne!(session_of(&mut db, other).await, outing);
        assert!(session_of(&mut db, between).await.is_some());
    }

    /// A flight between sessions a person changed joins one of them and
    /// merges nothing: a split stays split.
    #[sqlx::test]
    async fn a_split_stays_split(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let first = from_controller(&mut db, None, "06:00", "06:20").await;
        let second = from_controller(&mut db, None, "07:00", "07:20").await;
        assert_eq!(
            session_of(&mut db, first).await,
            session_of(&mut db, second).await
        );
        let general = crate::testing::general(&mut db).await;
        split(&mut db, &[second], general).await.unwrap().unwrap();

        let between = from_controller(&mut db, None, "06:30", "06:50").await;
        assert!(session_of(&mut db, between).await.is_some());
        assert_ne!(
            session_of(&mut db, first).await,
            session_of(&mut db, second).await
        );
        // Both halves count as changed, whichever a later flight is nearest.
        let unchanged: i64 =
            sqlx::query_scalar("select count(*) from sessions where changed_at is null")
                .fetch_one(&mut *db)
                .await
                .unwrap();
        assert_eq!(unchanged, 0);
    }

    /// A session someone commented on isn't merged by itself, also after the
    /// comment is deleted.
    #[sqlx::test]
    async fn a_commented_session_isnt_merged(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let author = crate::testing::user(&mut db, "A", &["admin"]).await;
        let actor = crate::testing::actor(&mut db, author).await;
        let early = from_controller(&mut db, None, "06:00", "06:20").await;
        let late = from_controller(&mut db, None, "10:00", "10:20").await;
        let session = session_of(&mut db, early).await.unwrap();
        let added = super::super::comments::add(&mut db, &actor, session, "Test comment").await;
        assert!(added.is_ok());
        sqlx::query("delete from session_comments")
            .execute(&mut *db)
            .await
            .unwrap();
        from_controller(&mut db, None, "08:00", "08:20").await;
        assert_ne!(
            session_of(&mut db, early).await,
            session_of(&mut db, late).await
        );
    }

    /// A pilot flying for two groups in one evening gets a session per group,
    /// each owned by its group.
    #[sqlx::test]
    async fn uploads_group_per_group(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let north = crate::testing::group(&mut db, "North").await;
        let south = crate::testing::group(&mut db, "South").await;
        let first = upload_for(&mut db, a, Some(north), "08:00", "08:20", "21:00").await;
        let other_group = upload_for(&mut db, a, Some(south), "08:30", "08:50", "21:00").await;
        let same_group = upload_for(&mut db, a, Some(north), "09:00", "09:20", "21:00").await;
        assert_ne!(
            session_of(&mut db, first).await,
            session_of(&mut db, other_group).await
        );
        assert_eq!(
            session_of(&mut db, first).await,
            session_of(&mut db, same_group).await
        );
        for (flight, group) in [(first, north), (other_group, south)] {
            let session = session_of(&mut db, flight).await.unwrap();
            let groups = groups_of(&mut db, &[session]).await.unwrap();
            assert_eq!(groups, Some(vec![group]));
        }
    }

    /// Merging keeps the session that starts first, and a later upload
    /// doesn't undo a split.
    #[sqlx::test]
    async fn merges_and_splits_stay(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let b = insert_id(
            &mut db,
            "insert into users (name) values ('B') returning id",
        )
        .await;
        let first = upload(&mut db, a, "08:00", "08:20", "10:00").await;
        let second = upload(&mut db, a, "08:30", "08:50", "10:00").await;
        let other_pilot = upload(&mut db, b, "07:30", "07:50", "10:00").await;

        let a_session = session_of(&mut db, first).await.unwrap();
        let b_session = session_of(&mut db, other_pilot).await.unwrap();
        let merged = merge(&mut db, &[a_session, b_session]).await.unwrap();
        assert_eq!(merged, b_session, "B's session starts first");
        assert_eq!(session_of(&mut db, first).await, Some(merged));

        let general = crate::testing::general(&mut db).await;
        let split_off = split(&mut db, &[second], general).await.unwrap().unwrap();
        // A new flight of A right after `second` joins one of the sessions
        // without moving the split flight back.
        upload(&mut db, a, "09:00", "09:20", "10:01").await;
        assert_eq!(session_of(&mut db, second).await, Some(split_off));
        assert_eq!(session_of(&mut db, first).await, Some(merged));
    }

    /// A window or a filter picks sessions with a flight in it, and they
    /// still add up all their flights, not only the ones that matched.
    #[sqlx::test]
    async fn window_and_filter_pick_whole_sessions(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let morning = upload(&mut db, a, "08:00", "08:20", "21:30").await;
        let evening = upload(&mut db, a, "09:30", "09:50", "21:30").await;
        assert_eq!(
            session_of(&mut db, morning).await,
            session_of(&mut db, evening).await
        );

        let everything = FlightScope {
            all: true,
            user_id: a,
            group_ids: Vec::new(),
        };
        let at =
            |time: &str| -> DateTime<Utc> { format!("2026-10-12T{time}:00Z").parse().unwrap() };
        let around_morning = Some(Window {
            from: at("07:00"),
            to: at("08:10"),
        });
        let all = Filter::default();
        let listed = list(&mut db, &everything, &all, around_morning, 10, 0, false)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].flights, 2);
        assert_eq!(
            count(&mut db, &everything, &all, around_morning)
                .await
                .unwrap(),
            1
        );

        let afternoon = Some(Window {
            from: at("12:00"),
            to: at("13:00"),
        });
        assert!(
            list(&mut db, &everything, &all, afternoon, 10, 0, false)
                .await
                .unwrap()
                .is_empty()
        );

        // Only the morning flight was with drone A.
        sqlx::query("update flights set aircraft_serial = 'A' where id = $1")
            .bind(morning)
            .execute(&mut *db)
            .await
            .unwrap();
        let drone = |serial: &str| Filter {
            aircraft_serials: Some(vec![serial.to_owned()]),
            ..Filter::default()
        };
        let listed = list(&mut db, &everything, &drone("A"), None, 10, 0, false)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].flights, 2);
        assert_eq!(
            count(&mut db, &everything, &drone("B"), None)
                .await
                .unwrap(),
            0
        );
    }

    /// A session across the end of a period is in the period of its first
    /// takeoff only, not in both (user's decision, 2026-11-12).
    #[sqlx::test]
    async fn sessions_are_in_the_period_they_start(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let first = upload(&mut db, a, "08:00", "08:20", "21:30").await;
        let second = upload(&mut db, a, "09:30", "09:50", "21:30").await;
        assert_eq!(
            session_of(&mut db, first).await,
            session_of(&mut db, second).await
        );
        let everything = FlightScope {
            all: true,
            user_id: a,
            group_ids: Vec::new(),
        };
        // Periods that change over at 09:00, between the two flights.
        let change: DateTime<Utc> = "2026-10-12T09:00:00Z".parse().unwrap();
        let before = Filter {
            to: Some(change),
            ..Filter::default()
        };
        let after = Filter {
            from: Some(change),
            ..Filter::default()
        };
        let listed = list(&mut db, &everything, &before, None, 10, 0, false)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].flights, 2);
        assert_eq!(count(&mut db, &everything, &before, None).await.unwrap(), 1);
        assert!(
            list(&mut db, &everything, &after, None, 10, 0, false)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(count(&mut db, &everything, &after, None).await.unwrap(), 0);
    }

    /// Stepping through sessions follows the list: newest first, and only
    /// the sessions that pass its filter.
    #[sqlx::test]
    async fn neighbours_follow_the_list(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let morning = upload(&mut db, a, "08:00", "08:20", "21:30").await;
        let noon = upload(&mut db, a, "12:00", "12:20", "21:30").await;
        let evening = upload(&mut db, a, "16:00", "16:20", "21:30").await;
        let everything = FlightScope {
            all: true,
            user_id: a,
            group_ids: Vec::new(),
        };
        let (morning, noon, evening) = (
            session_of(&mut db, morning).await.unwrap(),
            session_of(&mut db, noon).await.unwrap(),
            session_of(&mut db, evening).await.unwrap(),
        );
        let all = Filter::default();
        let place = neighbours(&mut db, &everything, &all, noon)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(place.previous, Some(evening), "newer first");
        assert_eq!(place.next, Some(morning));
        assert!(place.previous_start > place.next_start);
        let listed: Vec<Uuid> = list(&mut db, &everything, &all, None, 10, 0, false)
            .await
            .unwrap()
            .into_iter()
            .map(|session| session.id)
            .collect();
        assert_eq!(listed, [evening, noon, morning]);

        // Drone A flew in the morning and the evening, not at noon.
        sqlx::query("update flights set aircraft_serial = 'A' where session_id = any($1)")
            .bind([morning, evening])
            .execute(&mut *db)
            .await
            .unwrap();
        let drone_a = Filter {
            aircraft_serials: Some(vec!["A".to_owned()]),
            ..Filter::default()
        };
        let place = neighbours(&mut db, &everything, &drone_a, evening)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(place.next, Some(morning), "noon is left out");
        assert_eq!(place.previous, None);
        assert!(
            neighbours(&mut db, &everything, &drone_a, noon)
                .await
                .unwrap()
                .is_none()
        );
    }

    /// The cards' arrows go to the nearest sessions that pass the filter,
    /// placed by their first takeoff: one starting at the period's end is in
    /// the next period, and nothing beyond the ends.
    #[sqlx::test]
    async fn around_skips_to_the_nearest_sessions(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let morning = upload(&mut db, a, "08:00", "08:20", "21:30").await;
        upload(&mut db, a, "12:00", "12:20", "21:30").await;
        let evening = upload(&mut db, a, "16:00", "16:20", "21:30").await;
        let everything = FlightScope {
            all: true,
            user_id: a,
            group_ids: Vec::new(),
        };
        let at =
            |time: &str| -> DateTime<Utc> { format!("2026-10-12T{time}:00Z").parse().unwrap() };
        let period = |from: &str, to: &str| Filter {
            from: Some(at(from)),
            to: Some(at(to)),
            ..Filter::default()
        };

        let found = around(&mut db, &everything, &period("10:00", "16:00"))
            .await
            .unwrap();
        assert_eq!(found.before, Some(at("08:00")));
        assert_eq!(
            found.after,
            Some(at("16:00")),
            "starting at the end is after"
        );
        let found = around(&mut db, &everything, &period("06:00", "07:00"))
            .await
            .unwrap();
        assert_eq!(found.before, None, "nothing earlier");
        assert_eq!(found.after, Some(at("08:00")));
        let found = around(&mut db, &everything, &period("08:00", "09:00"))
            .await
            .unwrap();
        assert_eq!(found.before, None, "starting at the start is in it");

        // Someone who sees none of these flights is led nowhere.
        let b = insert_id(
            &mut db,
            "insert into users (name) values ('B') returning id",
        )
        .await;
        let own = FlightScope {
            all: false,
            user_id: b,
            group_ids: Vec::new(),
        };
        let found = around(&mut db, &own, &period("10:00", "11:00"))
            .await
            .unwrap();
        assert_eq!(
            found,
            Around {
                before: None,
                after: None
            }
        );

        // Drone A flew in the morning and the evening, not at noon.
        let mut drone_a = period("09:00", "10:00");
        drone_a.aircraft_serials = Some(vec!["A".to_owned()]);
        sqlx::query("update flights set aircraft_serial = 'A' where id = any($1)")
            .bind([morning, evening])
            .execute(&mut *db)
            .await
            .unwrap();
        let found = around(&mut db, &everything, &drone_a).await.unwrap();
        assert_eq!(found.after, Some(at("16:00")), "noon is skipped");
        drone_a.from = Some(at("17:00"));
        drone_a.to = Some(at("18:00"));
        let found = around(&mut db, &everything, &drone_a).await.unwrap();
        assert_eq!(
            found,
            Around {
                before: Some(at("16:00")),
                after: None
            }
        );
    }

    /// A session merged across groups shows each group only its own flights,
    /// in the list and the session's flights alike; outsiders see nothing.
    #[sqlx::test]
    async fn sessions_show_only_visible_flights(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let north = crate::testing::group(&mut db, "North").await;
        let south = crate::testing::group(&mut db, "South").await;
        let a = insert_id(
            &mut db,
            "insert into users (name) values ('A') returning id",
        )
        .await;
        let b = insert_id(
            &mut db,
            "insert into users (name) values ('B') returning id",
        )
        .await;
        let outsider = insert_id(
            &mut db,
            "insert into users (name) values ('C') returning id",
        )
        .await;
        let session = insert_id(&mut db, "insert into sessions (group_id) select id from groups where name = 'General' returning id").await;
        for (pilot, group, takeoff, landed) in
            [(a, north, "08:00", "08:20"), (b, south, "08:30", "08:50")]
        {
            crate::testing::join(&mut db, pilot, group).await;
            let flight = flight(&mut db, pilot, takeoff, landed).await;
            sqlx::query("update flights set group_id = $2, session_id = $3 where id = $1")
                .bind(flight)
                .bind(group)
                .bind(session)
                .execute(&mut *db)
                .await
                .unwrap();
        }

        let all = Filter::default();
        let member = crate::testing::scope(&mut db, a).await;
        let listed = list(&mut db, &member, &all, None, 10, 0, false)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            (listed[0].flights, listed[0].pilots.clone()),
            (1, vec!["A".to_owned()])
        );
        assert_eq!(count(&mut db, &member, &all, None).await.unwrap(), 1);
        let flights = crate::flights::db::of_session(&mut db, &member, session)
            .await
            .unwrap();
        assert_eq!(flights.len(), 1);

        let stranger = crate::testing::scope(&mut db, outsider).await;
        assert!(
            list(&mut db, &stranger, &all, None, 10, 0, false)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(count(&mut db, &stranger, &all, None).await.unwrap(), 0);
        let flights = crate::flights::db::of_session(&mut db, &stranger, session)
            .await
            .unwrap();
        assert!(flights.is_empty());

        let everything = FlightScope {
            all: true,
            user_id: outsider,
            group_ids: Vec::new(),
        };
        assert_eq!(
            list(&mut db, &everything, &all, None, 10, 0, false)
                .await
                .unwrap()[0]
                .flights,
            2
        );
    }
}
