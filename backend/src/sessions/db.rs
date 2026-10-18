use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::SessionGap;
use crate::policy::FlightScope;

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
}

/// One page of the sessions in the scope, newest first.
pub async fn list(
    db: &PgPool,
    scope: &FlightScope,
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
                array_remove(array_agg(distinct t.name), null) as "teams!"
         from flights f
         left join users u on u.id = f.pilot_id
         left join teams t on t.id = f.team_id
         where f.session_id is not null
           and ($1
                or f.pilot_id = $2
                or f.team_id in (select team_id from team_members where user_id = $2))
         group by f.session_id
         order by min(f.takeoff_at) desc, f.session_id
         limit $3 offset $4"#,
        scope.all,
        scope.user_id,
        limit,
        offset,
    )
    .fetch_all(db)
    .await
}

/// The number of sessions in the scope.
pub async fn count(db: &PgPool, scope: &FlightScope) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"select count(distinct session_id) as "count!"
           from flights
           where $1
              or pilot_id = $2
              or team_id in (select team_id from team_members where user_id = $2)"#,
        scope.all,
        scope.user_id,
    )
    .fetch_one(db)
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

async fn delete_empty(tx: &mut sqlx::PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        "delete from sessions s where not exists (select 1 from flights f where f.session_id = s.id)"
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Places every flight that has a pilot but no session, in takeoff order: it
/// joins the nearest session holding a flight of the same pilot less than the
/// gap apart, or else starts a new session. Flights already in a session stay
/// where they are, so this never undoes a user's changes. Deletes sessions
/// that have no flights left.
pub async fn group(db: &PgPool, gap: SessionGap) -> sqlx::Result<()> {
    let mut tx = locked(db).await?;
    let unplaced = sqlx::query!(
        r#"select id, pilot_id as "pilot_id!", takeoff_at, landed_at
           from flights
           where pilot_id is not null and session_id is null
           order by takeoff_at, id"#
    )
    .fetch_all(&mut *tx)
    .await?;

    for flight in unplaced {
        // The break between two flights; negative when they overlap.
        let joined = sqlx::query_scalar!(
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
        let session = match joined {
            Some(session) => session,
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
    }

    delete_empty(&mut tx).await?;
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

    async fn session_of(db: &PgPool, flight: Uuid) -> Option<Uuid> {
        sqlx::query_scalar("select session_id from flights where id = $1")
            .bind(flight)
            .fetch_one(db)
            .await
            .unwrap()
    }

    /// A pilot's flights less than the gap apart share a session, also when
    /// processed out of order; a longer break or another pilot starts a new one.
    #[sqlx::test]
    async fn groups_by_pilot_and_gap(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let gap = SessionGap(120);

        let first = flight(&db, a, "18:12", "18:40").await;
        let last = flight(&db, a, "20:30", "21:21").await;
        let other_pilot = flight(&db, b, "18:20", "18:50").await;
        group(&db, gap).await.unwrap();
        assert_eq!(session_of(&db, first).await, session_of(&db, last).await);
        assert_ne!(
            session_of(&db, first).await,
            session_of(&db, other_pilot).await
        );

        // Processed later, in between: it joins, and doesn't move the others.
        let middle = flight(&db, a, "19:30", "19:50").await;
        // More than two hours after the last landing.
        let next_day = flight(&db, a, "23:22", "23:40").await;
        group(&db, gap).await.unwrap();
        assert_eq!(session_of(&db, middle).await, session_of(&db, first).await);
        assert_ne!(
            session_of(&db, next_day).await,
            session_of(&db, first).await
        );
    }

    /// A flight taken out of its session is placed again, and a session left
    /// without flights is deleted.
    #[sqlx::test]
    async fn regroups_and_deletes_empty_sessions(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let gap = SessionGap(120);
        let flown_by_b = flight(&db, b, "08:00", "08:20").await;
        let wrongly_a = flight(&db, a, "08:30", "08:50").await;
        group(&db, gap).await.unwrap();
        let old_session = session_of(&db, wrongly_a).await;

        sqlx::query("update flights set pilot_id = $2, session_id = null where id = $1")
            .bind(wrongly_a)
            .bind(b)
            .execute(&db)
            .await
            .unwrap();
        group(&db, gap).await.unwrap();
        assert_eq!(
            session_of(&db, wrongly_a).await,
            session_of(&db, flown_by_b).await
        );
        let left: i64 = sqlx::query_scalar("select count(*) from sessions where id = $1")
            .bind(old_session)
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(left, 0);
    }

    /// Merging keeps the session that starts first; a split survives the next
    /// grouping, so automatic grouping never undoes it.
    #[sqlx::test]
    async fn merges_and_splits_stay(db: PgPool) {
        let a = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let b = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let gap = SessionGap(120);
        let first = flight(&db, a, "08:00", "08:20").await;
        let second = flight(&db, a, "08:30", "08:50").await;
        let other_pilot = flight(&db, b, "07:30", "07:50").await;
        group(&db, gap).await.unwrap();

        let a_session = session_of(&db, first).await.unwrap();
        let b_session = session_of(&db, other_pilot).await.unwrap();
        let merged = merge(&db, &[a_session, b_session]).await.unwrap();
        assert_eq!(merged, b_session, "B's session starts first");
        assert_eq!(session_of(&db, first).await, Some(merged));

        let split_off = split(&db, &[second]).await.unwrap();
        // A new flight of A right after `second` joins one of the sessions
        // without moving the split flight back.
        flight(&db, a, "09:00", "09:20").await;
        group(&db, gap).await.unwrap();
        assert_eq!(session_of(&db, second).await, Some(split_off));
        assert_eq!(session_of(&db, first).await, Some(merged));
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

        let member = FlightScope {
            all: false,
            user_id: a,
        };
        let listed = list(&db, &member, 10, 0).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            (listed[0].flights, listed[0].pilots.clone()),
            (1, vec!["A".to_owned()])
        );
        assert_eq!(count(&db, &member).await.unwrap(), 1);
        let flights = crate::flights::db::of_session(&db, &member, session)
            .await
            .unwrap();
        assert_eq!(flights.len(), 1);

        let stranger = FlightScope {
            all: false,
            user_id: outsider,
        };
        assert!(list(&db, &stranger, 10, 0).await.unwrap().is_empty());
        assert_eq!(count(&db, &stranger).await.unwrap(), 0);
        let flights = crate::flights::db::of_session(&db, &stranger, session)
            .await
            .unwrap();
        assert!(flights.is_empty());

        let everything = FlightScope {
            all: true,
            user_id: outsider,
        };
        assert_eq!(list(&db, &everything, 10, 0).await.unwrap()[0].flights, 2);
    }
}
