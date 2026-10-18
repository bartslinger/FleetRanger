//! Accounts: the people who can log in, shown as "users" in the web app.

pub mod db;
pub mod routes;
mod service;

use anyhow::{Result, bail};
use sqlx::PgPool;

/// The `add-account` command: lets someone log in with the given email address.
/// This is how the first account of an install is added, with `admin` so it can
/// manage the rest. For an existing account, `admin` makes it an admin.
pub async fn add(db: &PgPool, email: &str, admin: bool) -> Result<()> {
    let Some(email) = service::email(email) else {
        bail!("{email:?} is not an email address");
    };
    let role = if admin { "admin" } else { "member" };
    if db::insert(db, &email, role).await?.is_some() {
        tracing::info!(
            email,
            role,
            "account added; it is tied to a login on first use"
        );
    } else if admin {
        db::set_role_by_email(db, &email, role).await?;
        tracing::info!(email, "the existing account is now an admin");
    } else {
        tracing::info!(email, "an account with this email already exists");
    }
    Ok(())
}
