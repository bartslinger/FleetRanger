//! Accounts: the people who can log in.

pub mod db;

use anyhow::{Result, bail};
use sqlx::PgPool;

/// The `add-account` command: lets someone log in with the given email address.
/// This is how the first account of an install is added, with `admin` so it can
/// manage the rest. For an existing account, `admin` makes it an admin.
pub async fn add(db: &PgPool, email: &str, admin: bool) -> Result<()> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        bail!("{email:?} is not an email address");
    }
    let role = if admin { "admin" } else { "member" };
    if db::insert(db, &email, role).await? {
        tracing::info!(email, role, "account added; it is tied to a login on first use");
    } else if admin {
        db::set_role(db, &email, role).await?;
        tracing::info!(email, "the existing account is now an admin");
    } else {
        tracing::info!(email, "an account with this email already exists");
    }
    Ok(())
}
