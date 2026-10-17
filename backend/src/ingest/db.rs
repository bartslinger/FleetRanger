use std::collections::HashSet;

use sqlx::PgPool;

pub async fn known_paths(db: &PgPool) -> sqlx::Result<HashSet<String>> {
    let paths = sqlx::query_scalar!("select path from raw_logs")
        .fetch_all(db)
        .await?;
    Ok(paths.into_iter().collect())
}

/// Registers a raw log as pending. Returns false if a log with the same content
/// (or path) is already registered.
pub async fn insert_raw_log(
    db: &PgPool,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
) -> sqlx::Result<bool> {
    let inserted = sqlx::query_scalar!(
        "insert into raw_logs (sha256, path, size_bytes) values ($1, $2, $3)
         on conflict do nothing
         returning id",
        sha256,
        path,
        size_bytes,
    )
    .fetch_optional(db)
    .await?;
    Ok(inserted.is_some())
}
