//! Users: everyone, pilots, viewers and admins. A user flies for a team, can
//! log in, or both.

mod db;
pub mod routes;
mod service;

use anyhow::{Result, bail};
use sqlx::PgPool;

use db::UserFields;

/// The `add-user` command: adds a user who can log in with the given email
/// address. This is how the first user of an install is added, with `admin` so
/// they can manage the rest. For an existing login, `admin` makes it an admin.
pub async fn add(db: &PgPool, email: &str, name: &str, admin: bool) -> Result<()> {
    let Some(email) = service::email(email) else {
        bail!("{email:?} is not an email address");
    };
    let name = name.trim();
    if name.is_empty() {
        bail!("the name is empty");
    }
    if let Some(id) = db::find_by_email(db, &email).await? {
        if admin {
            db::make_admin(db, id).await?;
            tracing::info!(email, "the existing user is now an admin");
        } else {
            tracing::info!(email, "a user with this email already exists");
        }
        return Ok(());
    }
    let role = if admin { "admin" } else { "member" };
    let fields = UserFields {
        name,
        role,
        team_id: None,
        email: Some(&email),
    };
    db::insert(db, &fields).await?;
    tracing::info!(email, role, "user added; the login is tied on first use");
    Ok(())
}
