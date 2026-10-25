//! Comments on a session: a record of what happened, written with
//! `sessions.edit` on the session and changed only by their authors. See
//! "Comments" under "Sessions" in docs/architecture.md.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db;
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy,
};

/// The longest comment, in characters.
const MAX_LENGTH: usize = 4000;

#[derive(Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    /// When the author last changed it, if they did.
    pub edited_at: Option<DateTime<Utc>>,
}

/// A session's comments, oldest first.
pub async fn of_session(db: &PgPool, session: Uuid) -> sqlx::Result<Vec<Comment>> {
    sqlx::query_as!(
        Comment,
        "select c.id, c.author_id, u.name as author_name, c.body, c.created_at, c.edited_at
         from session_comments c
         join users u on u.id = c.author_id
         where c.session_id = $1
         order by c.created_at, c.id",
        session
    )
    .fetch_all(db)
    .await
}

/// Adds the actor's comment to a session, and returns it.
pub async fn add(db: &PgPool, actor: &Actor, session: Uuid, body: &str) -> ApiResult<Uuid> {
    let groups = db::groups_of(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    let body = text(body)?;
    Ok(sqlx::query_scalar!(
        "insert into session_comments (session_id, author_id, body) values ($1, $2, $3)
         returning id",
        session,
        actor.user_id,
        body
    )
    .fetch_one(db)
    .await?)
}

/// Changes the text of one of the actor's own comments. It is marked edited
/// only when the text changes.
pub async fn edit(
    db: &PgPool,
    actor: &Actor,
    session: Uuid,
    id: Uuid,
    body: &str,
) -> ApiResult<()> {
    own(db, actor, session, id).await?;
    let body = text(body)?;
    sqlx::query!(
        "update session_comments set body = $2, edited_at = now()
         where id = $1 and body <> $2",
        id,
        body
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Deletes one of the actor's own comments.
pub async fn delete(db: &PgPool, actor: &Actor, session: Uuid, id: Uuid) -> ApiResult<()> {
    own(db, actor, session, id).await?;
    sqlx::query!("delete from session_comments where id = $1", id)
        .execute(db)
        .await?;
    Ok(())
}

/// Authors change only their own comments, and only while they may still
/// change the session.
async fn own(db: &PgPool, actor: &Actor, session: Uuid, id: Uuid) -> ApiResult<()> {
    let comment = sqlx::query!(
        "select c.author_id, s.group_id from session_comments c
         join sessions s on s.id = c.session_id
         where c.id = $1 and c.session_id = $2",
        id,
        session
    )
    .fetch_optional(db)
    .await?
    .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &[comment.group_id])?;
    if comment.author_id != actor.user_id {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

/// A comment's text, trimmed: not empty and not too long.
fn text(body: &str) -> ApiResult<&str> {
    let body = body.trim();
    if body.is_empty() {
        return Err(ApiError::Invalid("Enter a comment.".to_owned()));
    }
    if body.chars().count() > MAX_LENGTH {
        return Err(ApiError::Invalid(
            "A comment can be at most 4,000 characters.".to_owned(),
        ));
    }
    Ok(body)
}
