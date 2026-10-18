use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Adds a pairing request, and deletes expired ones. Returns false if the code
/// is already taken, so the caller can try another.
pub async fn add_request(
    db: &PgPool,
    code: &str,
    token_sha256: &[u8],
    serial: &str,
    model: &str,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    sqlx::query!("delete from pairing_requests where expires_at < now()")
        .execute(&mut *tx)
        .await?;
    // A controller that asks again replaces its earlier request.
    sqlx::query!("delete from pairing_requests where serial = $1", serial)
        .execute(&mut *tx)
        .await?;
    let added = sqlx::query!(
        "insert into pairing_requests (code, token_sha256, serial, model, expires_at)
         values ($1, $2, $3, $4, $5)
         on conflict (code) do nothing",
        code,
        token_sha256,
        serial,
        model,
        expires_at,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    tx.commit().await?;
    Ok(added)
}

/// Keeps a waiting request with this token alive for another `seconds`, as
/// long as it hasn't expired. Returns false if there is no such request.
pub async fn keep_waiting(db: &PgPool, token_sha256: &[u8], seconds: f64) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update pairing_requests set expires_at = now() + make_interval(secs => $2)
         where token_sha256 = $1 and expires_at > now()",
        token_sha256,
        seconds,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// A controller waiting to be paired, without its code: approving still needs
/// the code shown on the controller.
#[derive(Serialize)]
pub struct Waiting {
    pub serial: String,
    pub model: String,
    pub expires_at: DateTime<Utc>,
}

/// The controllers waiting to be paired, most recent first.
pub async fn waiting(db: &PgPool) -> sqlx::Result<Vec<Waiting>> {
    sqlx::query_as!(
        Waiting,
        "select serial, model, expires_at from pairing_requests
         where expires_at > now()
         order by expires_at desc",
    )
    .fetch_all(db)
    .await
}

/// A pairing request as the approving user sees it.
#[derive(Serialize)]
pub struct Request {
    pub serial: String,
    pub model: String,
    pub expires_at: DateTime<Utc>,
    /// The team the controller joins when this user approves: theirs.
    pub team_name: Option<String>,
}

/// The request with this code, unless it has expired, as this user sees it.
pub async fn request(db: &PgPool, code: &str, user_id: Uuid) -> sqlx::Result<Option<Request>> {
    sqlx::query_as!(
        Request,
        r#"select serial, model, expires_at,
                (select t.name from team_members m join teams t on t.id = m.team_id
                 where m.user_id = $2) as "team_name?"
           from pairing_requests
           where code = $1 and expires_at > now()"#,
        code,
        user_id,
    )
    .fetch_optional(db)
    .await
}

/// Approves a request: the controller with its serial (registered now if it's
/// new) gets the request's token and joins the team. Returns the serial, or
/// None if there is no such request or it has expired.
pub async fn approve(
    db: &PgPool,
    code: &str,
    team_id: Option<Uuid>,
) -> sqlx::Result<Option<String>> {
    let mut tx = db.begin().await?;
    let Some(request) = sqlx::query!(
        "delete from pairing_requests where code = $1 and expires_at > now()
         returning serial, token_sha256",
        code,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Ok(None);
    };
    sqlx::query!(
        "insert into remote_controllers (serial, team_id, token_sha256) values ($1, $2, $3)
         on conflict (serial) do update set
             team_id = excluded.team_id,
             token_sha256 = excluded.token_sha256",
        request.serial,
        team_id,
        request.token_sha256,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(request.serial))
}

/// A paired controller, as it authenticates.
pub struct Paired {
    pub id: Uuid,
    pub serial: String,
}

pub async fn by_token(db: &PgPool, token_sha256: &[u8]) -> sqlx::Result<Option<Paired>> {
    sqlx::query_as!(
        Paired,
        "select id, serial from remote_controllers where token_sha256 = $1",
        token_sha256,
    )
    .fetch_optional(db)
    .await
}

/// Unpairs a controller: its token stops working. Returns false if there is no
/// such controller.
pub async fn unpair(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update remote_controllers set token_sha256 = null where id = $1",
        id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    /// Approving gives the controller the token whose hash it sent, in the
    /// approver's team; the request can't be approved twice, and an expired
    /// request can't be approved at all.
    #[sqlx::test]
    async fn approving_pairs_the_controller(db: PgPool) {
        let team: Uuid =
            sqlx::query_scalar("insert into teams (name) values ('North') returning id")
                .fetch_one(&db)
                .await
                .unwrap();
        let soon = Utc::now() + Duration::minutes(10);
        assert!(
            add_request(&db, "AAAA2222", b"token-hash", "RC1", "Test RC", soon)
                .await
                .unwrap()
        );
        assert!(
            !add_request(&db, "AAAA2222", b"other", "RC2", "Test RC", soon)
                .await
                .unwrap()
        );

        let serial = approve(&db, "AAAA2222", Some(team)).await.unwrap();
        assert_eq!(serial.as_deref(), Some("RC1"));
        assert!(
            approve(&db, "AAAA2222", Some(team))
                .await
                .unwrap()
                .is_none()
        );
        let paired = by_token(&db, b"token-hash").await.unwrap().unwrap();
        assert_eq!(paired.serial, "RC1");
        let team_of: Option<Uuid> =
            sqlx::query_scalar("select team_id from remote_controllers where id = $1")
                .bind(paired.id)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(team_of, Some(team));

        let past = Utc::now() - Duration::minutes(1);
        add_request(&db, "BBBB3333", b"late", "RC3", "Test RC", past)
            .await
            .unwrap();
        assert!(approve(&db, "BBBB3333", None).await.unwrap().is_none());
        // Checking can't revive an expired request, only keep a live one alive.
        assert!(!keep_waiting(&db, b"late", 60.0).await.unwrap());
        add_request(&db, "CCCC4444", b"live", "RC4", "Test RC", soon)
            .await
            .unwrap();
        assert!(keep_waiting(&db, b"live", 3600.0).await.unwrap());
        let expires: DateTime<Utc> =
            sqlx::query_scalar("select expires_at from pairing_requests where code = 'CCCC4444'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert!(expires > soon);

        assert!(unpair(&db, paired.id).await.unwrap());
        assert!(by_token(&db, b"token-hash").await.unwrap().is_none());
    }
}
