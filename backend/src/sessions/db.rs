use sqlx::PgPool;

use super::SessionGap;

/// Any constant, so that only one grouping runs at a time.
const GROUPING_LOCK: i64 = 0x5e55_1015;

/// Places every flight that has a pilot but no session, in takeoff order: it
/// joins the nearest session holding a flight of the same pilot less than the
/// gap apart, or else starts a new session. Flights already in a session stay
/// where they are, so this never undoes a user's changes. Deletes sessions
/// that have no flights left.
pub async fn group(db: &PgPool, gap: SessionGap) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query!("select pg_advisory_xact_lock($1)", GROUPING_LOCK)
        .execute(&mut *tx)
        .await?;

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

    sqlx::query!(
        "delete from sessions s where not exists (select 1 from flights f where f.session_id = s.id)"
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

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
}
