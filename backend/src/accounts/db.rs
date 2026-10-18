use sqlx::PgPool;

/// Adds an account for a lowercase email address. Returns false if one exists.
pub async fn insert(db: &PgPool, email: &str) -> sqlx::Result<bool> {
    let inserted = sqlx::query_scalar!(
        "insert into accounts (email) values ($1)
         on conflict (email) do nothing
         returning id",
        email,
    )
    .fetch_optional(db)
    .await?;
    Ok(inserted.is_some())
}
