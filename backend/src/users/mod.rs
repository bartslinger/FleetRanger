//! Users: everyone, pilots, viewers and admins. A user has roles in groups or
//! on the whole organisation, can log in, or both.

pub(crate) mod db;
pub mod routes;
mod service;

use anyhow::{Result, bail};
use sqlx::{Connection, PgConnection};

use db::UserFields;

/// The `add-user` command: adds a user who can log in with the given email
/// address. This is how the first user of an install is added, with `admin` so
/// they can manage the rest. For an existing login, `admin` makes it an admin.
pub async fn add(db: &mut PgConnection, email: &str, name: &str, admin: bool) -> Result<()> {
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
    let roles = if admin {
        vec!["admin".to_owned()]
    } else {
        Vec::new()
    };
    let fields = UserFields {
        name,
        roles: &roles,
        email: Some(&email),
        invited_by: None,
    };
    // In one transaction, so a failure can't leave the first admin without
    // the Pilot role while a retry finds them as an existing user.
    let mut tx = db.begin().await?;
    let id = match db::insert(&mut tx, &fields).await? {
        Ok(id) => id,
        Err(limit) => bail!("{} (see set-organisation --plan)", limit.message()),
    };
    // The organisation's first admin usually flies too, so they're in the
    // controller app's pilot list at once, as whoever signs up is (user's
    // decision, 2026-11-20).
    if admin {
        db::pilot_if_first(&mut tx, id).await?;
    }
    tx.commit().await?;
    tracing::info!(
        email,
        admin,
        "user added; they accept the invitation after logging in"
    );
    Ok(())
}
