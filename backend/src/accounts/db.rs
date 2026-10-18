use sqlx::PgPool;

/// Adds an account for a lowercase email address. Returns false if one exists.
pub async fn insert(db: &PgPool, email: &str, role: &str) -> sqlx::Result<bool> {
    let inserted = sqlx::query_scalar!(
        "insert into accounts (email, role) values ($1, $2)
         on conflict (email) do nothing
         returning id",
        email,
        role,
    )
    .fetch_optional(db)
    .await?;
    Ok(inserted.is_some())
}

pub async fn set_role(db: &PgPool, email: &str, role: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "update accounts set role = $2 where email = $1",
        email,
        role
    )
    .execute(db)
    .await?;
    Ok(())
}
