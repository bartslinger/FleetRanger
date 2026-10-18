//! Accounts: the people who can log in.

pub mod db;

use anyhow::{Result, bail};
use sqlx::PgPool;

/// The `add-account` command: lets someone log in with the given email address.
/// This is how the first account of an install is added.
pub async fn add(db: &PgPool, email: &str) -> Result<()> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        bail!("{email:?} is not an email address");
    }
    if db::insert(db, &email).await? {
        tracing::info!(email, "account added; it is tied to a login on first use");
    } else {
        tracing::info!(email, "an account with this email already exists");
    }
    Ok(())
}
