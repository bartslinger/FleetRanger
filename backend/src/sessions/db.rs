use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{LATE_UPLOAD_HOURS, SessionGap};
use crate::{
    flights::db::{Filter, Neighbours},
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
    pub teams: Vec<String>,
    /// So the web app offers merging only where the policy allows it.
    pub team_ids: Vec<Uuid>,
}

/// A time span: sessions with a flight in it, e.g. the ones near flights to be
/// added to a session.
#[derive(Clone, Copy)]
pub struct Window {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

// The Sessions list's filters are the logbook's: a session passes when one of
// its visible flights does. Its numbers stay those of all its visible flights.

/// One page of the sessions in the scope that pass the filter (and have a
/// flight in the window, if given), newest first.
pub async fn list(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    window: Option<Window>,
    limit: i64,
    offset: i64,
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
                array_remove(array_agg(distinct t.name), null) as "teams!",
                array_remove(array_agg(distinct f.team_id), null) as "team_ids!"
         from flights f
         left join users u on u.id = f.pilot_id
         left join teams t on t.id = f.team_id
         where f.session_id is not null
           and ($1
                or f.pilot_id = $2
                or f.team_id in (select team_id from team_members where user_id = $2))
           and ($5::timestamptz is null
                or f.session_id in (select session_id from flights
                                    where takeoff_at < $6 and landed_at > $5))
           and f.session_id in (
               select g.session_id from flights g
               where g.session_id is not null
                 and ($1
                      or g.pilot_id = $2
                      or g.team_id in (select team_id from team_members where user_id = $2))
                 and ($7::text[] is null or g.aircraft_serial = any($7))
                 and ($8::text[] is null or g.rc_serial = any($8))
                 and ($9::timestamptz is null or g.takeoff_at >= $9)
                 and ($10::timestamptz is null or g.takeoff_at < $10)
                 and ($11::uuid[] is null or g.team_id = any($11)
                      or ($12 and g.team_id is null)))
         group by f.session_id
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
        filter.team_ids.as_deref(),
        filter.queue,
    )
    .fetch_all(db)
    .await
}

/// The number of sessions `list` lists in all.
pub async fn count(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    window: Option<Window>,
) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"select count(distinct session_id) as "count!"
           from flights
           where ($1
                  or pilot_id = $2
                  or team_id in (select team_id from team_members where user_id = $2))
             and ($3::timestamptz is null
                  or session_id in (select session_id from flights
                                    where takeoff_at < $4 and landed_at > $3))
             and session_id in (
                 select g.session_id from flights g
                 where g.session_id is not null
                   and ($1
                        or g.pilot_id = $2
                        or g.team_id in (select team_id from team_members where user_id = $2))
                   and ($5::text[] is null or g.aircraft_serial = any($5))
                   and ($6::text[] is null or g.rc_serial = any($6))
                   and ($7::timestamptz is null or g.takeoff_at >= $7)
                   and ($8::timestamptz is null or g.takeoff_at < $8)
                   and ($9::uuid[] is null or g.team_id = any($9)
                        or ($10 and g.team_id is null)))"#,
        scope.all,
        scope.user_id,
        window.map(|w| w.from),
        window.map(|w| w.to),
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.team_ids.as_deref(),
        filter.queue,
    )
    .fetch_one(db)
    .await
}

/// Where a session is in the scope's list with this filter, newest first as
/// `list` orders it: the sessions before and after it, its position (from 1)
/// and the list's length. None if the session isn't in that list.
pub async fn neighbours(
    db: &PgPool,
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
                      or f.team_id in (select team_id from team_members where user_id = $2))
                 and f.session_id in (
                     select g.session_id from flights g
                     where g.session_id is not null
                       and ($1
                            or g.pilot_id = $2
                            or g.team_id in (select team_id from team_members where user_id = $2))
                       and ($4::text[] is null or g.aircraft_serial = any($4))
                       and ($5::text[] is null or g.rc_serial = any($5))
                       and ($6::timestamptz is null or g.takeoff_at >= $6)
                       and ($7::timestamptz is null or g.takeoff_at < $7)
                       and ($8::uuid[] is null or g.team_id = any($8)
                            or ($9 and g.team_id is null)))
               group by f.session_id
           ),
           placed as (
               select id,
                      lag(id) over listing as previous,
                      lead(id) over listing as next,
                      row_number() over listing as position,
                      count(*) over () as total
               from listed
               window listing as (order by start desc, id)
           )
           select previous, next, position as "position!", total as "total!"
           from placed
           where id = $3"#,
        scope.all,
        scope.user_id,
        id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.from,
        filter.to,
        filter.team_ids.as_deref(),
        filter.queue,
    )
    .fetch_optional(db)
    .await
}

/// Any constant, so that only one change to sessions (grouping, splitting,
/// merging) runs at a time.
const SESSIONS_LOCK: i64 = 0x5e55_1015;

/// A transaction holding the sessions lock.
async fn locked(db: &PgPool) -> sqlx::Result<sqlx::Transaction<'static, sqlx::Postgres>> {
    let mut tx = db.begin().await?;
    sqlx::query!("select pg_advisory_xact_lock($1)", SESSIONS_LOCK)
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

/// The team of every flight in these sessions, or None if one of them doesn't
/// exist. The visibility check is the policy's job, not this query's.
pub async fn flight_teams(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Option<Vec<Option<Uuid>>>> {
    let flights = sqlx::query!(
        r#"select session_id as "session_id!", team_id from flights where session_id = any($1)"#,
        ids
    )
    .fetch_all(db)
    .await?;
    let found: HashSet<Uuid> = flights.iter().map(|flight| flight.session_id).collect();
    Ok((found.len() == ids.len())
        .then(|| flights.into_iter().map(|flight| flight.team_id).collect()))
}

/// The one session all these flights are in, or None if one of them doesn't
/// exist, isn't in a session, or they are in different sessions.
pub async fn session_of(db: &PgPool, flight_ids: &[Uuid]) -> sqlx::Result<Option<Uuid>> {
    let sessions = sqlx::query_scalar!(
        "select session_id from flights where id = any($1)",
        flight_ids
    )
    .fetch_all(db)
    .await?;
    let first = sessions.first().copied().flatten();
    let same = sessions.len() == flight_ids.len() && sessions.iter().all(|s| *s == first);
    Ok(first.filter(|_| same))
}

/// Puts all flights of these sessions in the one that starts first, deletes
/// the others, and returns the remaining session.
pub async fn merge(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Uuid> {
    let mut tx = locked(db).await?;
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
        "delete from sessions where id = any($1) and id <> $2",
        ids,
        target
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(target)
}

/// Moves these flights into a new session, and returns it. Deletes a session
/// left without flights.
pub async fn split(db: &PgPool, flight_ids: &[Uuid]) -> sqlx::Result<Uuid> {
    let mut tx = locked(db).await?;
    let session = sqlx::query_scalar!("insert into sessions default values returning id")
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query!(
        "update flights set session_id = $2 where id = any($1)",
        flight_ids,
        session
    )
    .execute(&mut *tx)
    .await?;
    delete_empty(&mut tx).await?;
    tx.commit().await?;
    Ok(session)
}

/// Moves these flights into the session. Deletes a session left without flights.
pub async fn add(db: &PgPool, session: Uuid, flight_ids: &[Uuid]) -> sqlx::Result<()> {
    let mut tx = locked(db).await?;
    sqlx::query!(
        "update flights set session_id = $2 where id = any($1)",
        flight_ids,
        session
    )
    .execute(&mut *tx)
    .await?;
    delete_empty(&mut tx).await?;
    tx.commit().await
}

/// Takes these flights out of their sessions, and returns how many were in
/// one. Deletes a session left without flights.
pub async fn remove(db: &PgPool, flight_ids: &[Uuid]) -> sqlx::Result<u64> {
    let mut tx = locked(db).await?;
    let removed = sqlx::query!(
        "update flights set session_id = null where id = any($1) and session_id is not null",
        flight_ids
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    delete_empty(&mut tx).await?;
    tx.commit().await?;
    Ok(removed)
}

async fn delete_empty(tx: &mut sqlx::PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        "delete from sessions s where not exists (select 1 from flights f where f.session_id = s.id)"
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Places the flight of a log uploaded with its pilot, the one case where a
/// flight joins a session by itself. It joins the nearest session holding a
/// flight of the same pilot less than the gap apart, if that session's flights
/// came in less than [`LATE_UPLOAD_HOURS`] before it; a later one stays
/// without a session, for someone to add. With no such session, it starts a
/// new one. Imported logs have no pilot and flights already in a session stay
/// where they are, so this never undoes a user's changes.
pub async fn place_upload(db: &PgPool, raw_log_id: Uuid, gap: SessionGap) -> sqlx::Result<()> {
    let mut tx = locked(db).await?;
    let Some(flight) = sqlx::query!(
        r#"select f.id, f.pilot_id as "pilot_id!", f.takeoff_at, f.landed_at, l.received_at
           from flights f
           join raw_logs l on l.id = f.raw_log_id
           where f.raw_log_id = $1 and f.session_id is null
             and f.pilot_id is not null and l.pilot_id is not null"#,
        raw_log_id
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return tx.commit().await;
    };

    // The break between two flights; negative when they overlap.
    let nearest = sqlx::query_scalar!(
        r#"select session_id as "session_id!"
           from flights
           where pilot_id = $1 and session_id is not null
             and takeoff_at < $3::timestamptz + make_interval(mins => $4)
             and landed_at > $2::timestamptz - make_interval(mins => $4)
           order by greatest(takeoff_at - $3::timestamptz, $2::timestamptz - landed_at), id
           limit 1"#,
        flight.pilot_id,
        flight.takeoff_at,
        flight.landed_at,
        gap.0,
    )
    .fetch_optional(&mut *tx)
    .await?;
    let session = match nearest {
        Some(session) => {
            let same_sync = sqlx::query_scalar!(
                r#"select max(l.received_at) > $2::timestamptz - make_interval(hours => $3)
                       as "same_sync!"
                   from flights f
                   join raw_logs l on l.id = f.raw_log_id
                   where f.session_id = $1"#,
                session,
                flight.received_at,
                LATE_UPLOAD_HOURS,
            )
            .fetch_one(&mut *tx)
            .await?;
            if !same_sync {
                return tx.commit().await;
            }
            session
        }
        None => {
            sqlx::query_scalar!("insert into sessions default values returning id")
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

    async fn insert_id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    /// Adds a flight of this pilot, from and to times on the same day.
    async fn flight(db: &PgPool, pilot: Uuid, takeoff: &str, landed: &str) -> Uuid {
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
                                  track, pilot_id)
             values ($1, $2, $3, 0, 0, '{}', $4)
             returning id",
        )
        .bind(raw_log)
        .bind(at(takeoff))
        .bind(at(landed))
        .bind(pilot)
        .fetch_one(db)
        .await
        .unwrap()
    }

    /// Adds a flight of this pilot as a controller uploads it (its log carries
    /// the pilot), received on 12 Oct at `received` or on a later day's
    /// `"+1 12:00"`, and places it as the processing loop does.
    async fn upload(db: &PgPool, pilot: Uuid, takeoff: &str, landed: &str, received: &str) -> Uuid {
        let flight = flight(db, pilot, takeoff, landed).await;
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
        .fetch_one(db)
        .await
        .unwrap();
        place_upload(db, raw_log, SessionGap(120)).await.unwrap();
        flight
    }

    async fn session_of(db: &PgPool, flight: Uuid) -> Option<Uuid> {
        sqlx::query_scalar("select session_id from flights where id = $1")
            .bind(flight)
            .fetch_one(db)
            .await
            .unwrap()
    }

    /// Uploaded flights of a pilot less than the gap apart share a session,
    /// also when processed out of order; a longer break or another pilot
    /// starts a new one.
    #[sqlx::test]
    async fn uploads_group_by_pilot_and_gap(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;

        let first = upload(&db, a, "18:12", "18:40", "21:30").await;
        let last = upload(&db, a, "20:30", "21:21", "21:30").await;
        let other_pilot = upload(&db, b, "18:20", "18:50", "21:30").await;
        assert_eq!(session_of(&db, first).await, session_of(&db, last).await);
        assert_ne!(
            session_of(&db, first).await,
            session_of(&db, other_pilot).await
        );

        // Processed later, in between: it joins, and doesn't move the others.
        let middle = upload(&db, a, "19:30", "19:50", "21:31").await;
        assert_eq!(session_of(&db, middle).await, session_of(&db, first).await);
        // More than two hours after the last landing.
        let later = upload(&db, a, "23:22", "23:40", "23:50").await;
        assert!(session_of(&db, later).await.is_some());
        assert_ne!(session_of(&db, later).await, session_of(&db, first).await);
    }

    /// A flight that comes in long after its session's flights doesn't join
    /// it by itself, and neither do imported flights, which have no pilot from
    /// a controller: someone adds them.
    #[sqlx::test]
    async fn late_uploads_and_imports_stay_out(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let synced = upload(&db, a, "18:12", "18:40", "21:30").await;
        assert!(session_of(&db, synced).await.is_some());

        let late = upload(&db, a, "19:00", "19:20", "+1 12:00").await;
        assert_eq!(session_of(&db, late).await, None);

        let imported = flight(&db, a, "19:30", "19:50").await;
        let raw_log: Uuid = sqlx::query_scalar("select raw_log_id from flights where id = $1")
            .bind(imported)
            .fetch_one(&db)
            .await
            .unwrap();
        place_upload(&db, raw_log, SessionGap(120)).await.unwrap();
        assert_eq!(session_of(&db, imported).await, None);
    }

    /// Merging keeps the session that starts first, and a later upload
    /// doesn't undo a split.
    #[sqlx::test]
    async fn merges_and_splits_stay(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let first = upload(&db, a, "08:00", "08:20", "10:00").await;
        let second = upload(&db, a, "08:30", "08:50", "10:00").await;
        let other_pilot = upload(&db, b, "07:30", "07:50", "10:00").await;

        let a_session = session_of(&db, first).await.unwrap();
        let b_session = session_of(&db, other_pilot).await.unwrap();
        let merged = merge(&db, &[a_session, b_session]).await.unwrap();
        assert_eq!(merged, b_session, "B's session starts first");
        assert_eq!(session_of(&db, first).await, Some(merged));

        let split_off = split(&db, &[second]).await.unwrap();
        // A new flight of A right after `second` joins one of the sessions
        // without moving the split flight back.
        upload(&db, a, "09:00", "09:20", "10:01").await;
        assert_eq!(session_of(&db, second).await, Some(split_off));
        assert_eq!(session_of(&db, first).await, Some(merged));
    }

    /// A window or a filter picks sessions with a flight in it, and they
    /// still add up all their flights, not only the ones that matched.
    #[sqlx::test]
    async fn window_and_filter_pick_whole_sessions(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let morning = upload(&db, a, "08:00", "08:20", "21:30").await;
        let evening = upload(&db, a, "09:30", "09:50", "21:30").await;
        assert_eq!(
            session_of(&db, morning).await,
            session_of(&db, evening).await
        );

        let everything = FlightScope {
            all: true,
            user_id: a,
        };
        let at =
            |time: &str| -> DateTime<Utc> { format!("2026-10-12T{time}:00Z").parse().unwrap() };
        let around_morning = Some(Window {
            from: at("07:00"),
            to: at("08:10"),
        });
        let all = Filter::default();
        let listed = list(&db, &everything, &all, around_morning, 10, 0)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].flights, 2);
        assert_eq!(
            count(&db, &everything, &all, around_morning).await.unwrap(),
            1
        );

        let afternoon = Some(Window {
            from: at("12:00"),
            to: at("13:00"),
        });
        assert!(
            list(&db, &everything, &all, afternoon, 10, 0)
                .await
                .unwrap()
                .is_empty()
        );

        // Only the morning flight was with drone A.
        sqlx::query("update flights set aircraft_serial = 'A' where id = $1")
            .bind(morning)
            .execute(&db)
            .await
            .unwrap();
        let drone = |serial: &str| Filter {
            aircraft_serials: Some(vec![serial.to_owned()]),
            ..Filter::default()
        };
        let listed = list(&db, &everything, &drone("A"), None, 10, 0)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].flights, 2);
        assert_eq!(count(&db, &everything, &drone("B"), None).await.unwrap(), 0);
    }

    /// Stepping through sessions follows the list: newest first, and only
    /// the sessions that pass its filter.
    #[sqlx::test]
    async fn neighbours_follow_the_list(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let morning = upload(&db, a, "08:00", "08:20", "21:30").await;
        let noon = upload(&db, a, "12:00", "12:20", "21:30").await;
        let evening = upload(&db, a, "16:00", "16:20", "21:30").await;
        let everything = FlightScope {
            all: true,
            user_id: a,
        };
        let (morning, noon, evening) = (
            session_of(&db, morning).await.unwrap(),
            session_of(&db, noon).await.unwrap(),
            session_of(&db, evening).await.unwrap(),
        );
        let all = Filter::default();
        let place = neighbours(&db, &everything, &all, noon)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(place.previous, Some(evening), "newer first");
        assert_eq!(place.next, Some(morning));
        assert_eq!((place.position, place.total), (2, 3));
        let listed: Vec<Uuid> = list(&db, &everything, &all, None, 10, 0)
            .await
            .unwrap()
            .into_iter()
            .map(|session| session.id)
            .collect();
        assert_eq!(listed, [evening, noon, morning]);

        // Drone A flew in the morning and the evening, not at noon.
        sqlx::query("update flights set aircraft_serial = 'A' where session_id = any($1)")
            .bind([morning, evening])
            .execute(&db)
            .await
            .unwrap();
        let drone_a = Filter {
            aircraft_serials: Some(vec!["A".to_owned()]),
            ..Filter::default()
        };
        let place = neighbours(&db, &everything, &drone_a, evening)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(place.next, Some(morning), "noon is left out");
        assert_eq!((place.position, place.total), (1, 2));
        assert!(
            neighbours(&db, &everything, &drone_a, noon)
                .await
                .unwrap()
                .is_none()
        );
    }

    /// A session merged across teams shows each team only its own flights,
    /// in the list and the session's flights alike; outsiders see nothing.
    #[sqlx::test]
    async fn sessions_show_only_visible_flights(db: PgPool) {
        let north = insert_id(
            &db,
            "insert into teams (name) values ('North') returning id",
        )
        .await;
        let south = insert_id(
            &db,
            "insert into teams (name) values ('South') returning id",
        )
        .await;
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let outsider = insert_id(&db, "insert into users (name) values ('C') returning id").await;
        let session = insert_id(&db, "insert into sessions default values returning id").await;
        for (pilot, team, takeoff, landed) in
            [(a, north, "08:00", "08:20"), (b, south, "08:30", "08:50")]
        {
            sqlx::query("insert into team_members (user_id, team_id) values ($1, $2)")
                .bind(pilot)
                .bind(team)
                .execute(&db)
                .await
                .unwrap();
            let flight = flight(&db, pilot, takeoff, landed).await;
            sqlx::query("update flights set team_id = $2, session_id = $3 where id = $1")
                .bind(flight)
                .bind(team)
                .bind(session)
                .execute(&db)
                .await
                .unwrap();
        }

        let all = Filter::default();
        let member = FlightScope {
            all: false,
            user_id: a,
        };
        let listed = list(&db, &member, &all, None, 10, 0).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            (listed[0].flights, listed[0].pilots.clone()),
            (1, vec!["A".to_owned()])
        );
        assert_eq!(count(&db, &member, &all, None).await.unwrap(), 1);
        let flights = crate::flights::db::of_session(&db, &member, session)
            .await
            .unwrap();
        assert_eq!(flights.len(), 1);

        let stranger = FlightScope {
            all: false,
            user_id: outsider,
        };
        assert!(
            list(&db, &stranger, &all, None, 10, 0)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(count(&db, &stranger, &all, None).await.unwrap(), 0);
        let flights = crate::flights::db::of_session(&db, &stranger, session)
            .await
            .unwrap();
        assert!(flights.is_empty());

        let everything = FlightScope {
            all: true,
            user_id: outsider,
        };
        assert_eq!(
            list(&db, &everything, &all, None, 10, 0).await.unwrap()[0].flights,
            2
        );
    }
}
