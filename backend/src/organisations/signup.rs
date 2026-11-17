//! Self-service sign-up (`FLEETRANGER_SIGNUP=on`, user's decisions,
//! 2026-11-13): anyone with a Google account creates an organisation and
//! becomes its admin, until the cap is reached; after that, their email goes
//! on the waiting list, which the operator works through by hand. Someone
//! already admin of an organisation can't sign up another.

use sqlx::PgConnection;
use uuid::Uuid;

use crate::users::db::{UserFields, insert as insert_user};

/// What someone signing up chose on the sign-up page.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub organisation_name: String,
    pub time_zone: String,
}

/// The person Google vouches for.
pub struct Person<'a> {
    pub subject: &'a str,
    /// Verified by Google.
    pub email: &'a str,
    pub name: &'a str,
}

pub enum Outcome {
    Created {
        /// The new admin; the login session is theirs now.
        #[cfg_attr(
            not(test),
            expect(dead_code, reason = "the session says it; tests check it")
        )]
        user_id: Uuid,
        organisation_id: Uuid,
    },
    /// The cap is reached; their email is on the waiting list.
    Full,
    /// They are already admin of an organisation: one per person signs up
    /// (user's decision, 2026-11-16), against collecting free organisations.
    AlreadyAdmin,
}

/// An organisation this Google account is admin of, the oldest if several
/// (with `System`).
pub async fn admin_of(
    db: &mut PgConnection,
    subject: &str,
) -> sqlx::Result<Option<(Uuid, String)>> {
    let row = sqlx::query!(
        "select o.id, o.name from logins l
         join users u on u.id = l.user_id
         join organisation_roles r on r.user_id = l.user_id
         join organisations o on o.id = l.organisation_id
         where l.oidc_subject = $1 and r.role = 'admin'
           and u.archived_at is null and o.archived_at is null
         order by o.created_at
         limit 1",
        subject,
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(row.map(|row| (row.id, row.name)))
}

/// The new admin's name: as Google gives it, or the email when that is
/// blank; at most as long as an organisation name.
pub fn person_name(google: Option<&str>, email: &str) -> String {
    let name = google.map(str::trim).filter(|name| !name.is_empty());
    name.unwrap_or(email)
        .chars()
        .take(super::MAX_NAME)
        .collect()
}

/// Whether `cap` organisations that aren't archived exist, so that signing
/// up puts people on the waiting list (with `System`).
pub async fn full(db: &mut PgConnection, cap: i64) -> sqlx::Result<bool> {
    let organisations = sqlx::query_scalar!(
        r#"select count(*) as "count!" from organisations where archived_at is null"#
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(organisations >= cap)
}

/// Puts someone on the waiting list, which the operator works through by
/// hand (with `System`).
pub async fn join_waitlist(db: &mut PgConnection, email: &str, name: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into signup_waitlist (email, name) values (lower($1), $2)
         on conflict (email) do nothing",
        email.trim(),
        name,
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

/// Creates the organisation with the person as its admin, and puts their
/// login `session` in it, in one transaction; or puts them on the waiting
/// list when it is `full` (with `System`, as no organisation is known yet).
pub async fn sign_up(
    db: &mut PgConnection,
    request: &Request,
    person: &Person<'_>,
    cap: i64,
    session: Uuid,
) -> sqlx::Result<Outcome> {
    let email = person.email.trim().to_lowercase();
    let mut tx = sqlx::Connection::begin(&mut *db).await?;
    // One sign-up at a time, so two can't both take the last place.
    sqlx::query!("select pg_advisory_xact_lock(hashtext('fleetranger.signup'))")
        .execute(&mut *tx)
        .await?;
    if admin_of(&mut tx, person.subject).await?.is_some() {
        return Ok(Outcome::AlreadyAdmin);
    }
    if full(&mut tx, cap).await? {
        join_waitlist(&mut tx, &email, person.name).await?;
        tx.commit().await?;
        return Ok(Outcome::Full);
    }

    let organisation_id = super::insert(
        &mut tx,
        &request.organisation_name,
        super::Plan::Free,
        Some(&request.time_zone),
        None,
        None,
    )
    .await?;
    // The users tables take their organisation from the connection.
    sqlx::query("select set_config('app.organisation_id', $1, true)")
        .bind(organisation_id.to_string())
        .execute(&mut *tx)
        .await?;
    let Ok(user_id) = insert_user(
        &mut tx,
        &UserFields {
            name: person.name,
            roles: &["admin".to_owned()],
            email: Some(&email),
            invited_by: None,
        },
    )
    .await?
    else {
        // A new organisation's first login always fits its plan; rolled back
        // with the transaction all the same.
        return Err(sqlx::Error::RowNotFound);
    };
    sqlx::query!(
        "update logins set oidc_subject = $2 where user_id = $1",
        user_id,
        person.subject,
    )
    .execute(&mut *tx)
    .await?;
    // In now: off the waiting list the operator works through.
    sqlx::query!("delete from signup_waitlist where email = $1", email)
        .execute(&mut *tx)
        .await?;
    if !crate::auth::db::switch_organisation(&mut tx, session, organisation_id).await? {
        // Rolled back with the transaction: nothing half done.
        return Err(sqlx::Error::RowNotFound);
    }
    tx.commit().await?;
    Ok(Outcome::Created {
        user_id,
        organisation_id,
    })
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::{Outcome, Person, Request, sign_up};
    use crate::auth::db::{Account, Membership, create_session, memberships};

    /// A login session of someone in no organisation yet.
    async fn logged_in(db: &mut sqlx::PgConnection, subject: &str, token: u8) -> uuid::Uuid {
        let account = Account {
            subject,
            email: None,
            name: None,
        };
        let ip = [127, 0, 0, 1].into();
        create_session(db, &account, None, &[token; 32], 30, ip, None)
            .await
            .unwrap()
    }

    /// Signing up makes the person the admin of a new organisation, which
    /// they log in to, and they can't sign up another; once the cap is
    /// reached, the next sign-up creates nothing and puts their email on the
    /// waiting list.
    #[sqlx::test]
    async fn sign_up_creates_an_admin_until_the_cap_then_waits(pool: PgPool) {
        let mut db = pool.acquire().await.unwrap();
        let request = Request {
            organisation_name: "Test Org".to_owned(),
            time_zone: "Europe/Amsterdam".to_owned(),
        };
        let first = Person {
            subject: "test-subject-1",
            email: "Test-One@example.org",
            name: "Test One",
        };
        let session = logged_in(&mut db, "test-subject-1", 1).await;
        let Outcome::Created {
            user_id,
            organisation_id,
        } = sign_up(&mut db, &request, &first, 1, session)
            .await
            .unwrap()
        else {
            panic!("the first sign-up within the cap should create an organisation");
        };
        let admin: bool = sqlx::query_scalar(
            "select exists (select 1 from organisation_roles
                            where user_id = $1 and organisation_id = $2 and role = 'admin')",
        )
        .bind(user_id)
        .bind(organisation_id)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        assert!(admin);
        let found = memberships(&mut db, "test-subject-1").await.unwrap();
        assert_eq!(
            found,
            [Membership {
                user_id,
                organisation_id
            }],
            "the new admin should log in"
        );
        // Signing up again is refused: one organisation per admin.
        assert!(matches!(
            sign_up(&mut db, &request, &first, 10, session)
                .await
                .unwrap(),
            Outcome::AlreadyAdmin
        ));

        let second = Person {
            subject: "test-subject-2",
            email: "test-two@example.org",
            name: "Test Two",
        };
        let second_session = logged_in(&mut db, "test-subject-2", 2).await;
        assert!(matches!(
            sign_up(&mut db, &request, &second, 1, second_session)
                .await
                .unwrap(),
            Outcome::Full
        ));
        let organisations: i64 = sqlx::query_scalar("select count(*) from organisations")
            .fetch_one(&mut *db)
            .await
            .unwrap();
        assert_eq!(organisations, 1);
        let waiting: Vec<String> = sqlx::query_scalar("select email from signup_waitlist")
            .fetch_all(&mut *db)
            .await
            .unwrap();
        assert_eq!(waiting, ["test-two@example.org"]);
    }

    /// Someone who is only a member elsewhere may sign up their own
    /// organisation (the one-organisation rule is about admins), and their
    /// login moves to it in the same transaction; a move that fails creates
    /// nothing.
    #[sqlx::test]
    async fn a_member_elsewhere_signs_up_and_moves_there(pool: PgPool) {
        let mut db = pool.acquire().await.unwrap();
        let request = Request {
            organisation_name: "Test Org".to_owned(),
            time_zone: "UTC".to_owned(),
        };
        let admin = Person {
            subject: "test-subject-admin",
            email: "test-admin@example.org",
            name: "Test Admin",
        };
        let admin_session = logged_in(&mut db, "test-subject-admin", 3).await;
        let Outcome::Created {
            organisation_id, ..
        } = sign_up(&mut db, &request, &admin, 10, admin_session)
            .await
            .unwrap()
        else {
            panic!("the first sign-up should create an organisation");
        };
        let member: uuid::Uuid = sqlx::query_scalar(
            "insert into users (organisation_id, name) values ($1, 'Test Member') returning id",
        )
        .bind(organisation_id)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        sqlx::query(
            "insert into logins (organisation_id, user_id, email, oidc_subject)
             values ($1, $2, 'test-member@example.org', 'test-subject-member')",
        )
        .bind(organisation_id)
        .bind(member)
        .execute(&mut *db)
        .await
        .unwrap();

        let session: uuid::Uuid = sqlx::query_scalar(
            "insert into login_sessions (organisation_id, user_id, subject, token_sha256, expires_at, ip)
             values ($1, $2, 'test-subject-member', sha256('test'), now() + interval '1 day', '127.0.0.1')
             returning id",
        )
        .bind(organisation_id)
        .bind(member)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        let person = Person {
            subject: "test-subject-member",
            email: "test-member@example.org",
            name: "Test Member",
        };
        let count = async |db: &mut sqlx::PgConnection| -> i64 {
            sqlx::query_scalar("select count(*) from organisations")
                .fetch_one(db)
                .await
                .unwrap()
        };

        // A login session that isn't theirs can't be moved: nothing is made.
        assert!(
            sign_up(&mut db, &request, &person, 10, uuid::Uuid::nil())
                .await
                .is_err()
        );
        assert_eq!(count(&mut db).await, 1);

        let Outcome::Created {
            user_id,
            organisation_id: created,
        } = sign_up(&mut db, &request, &person, 10, session)
            .await
            .unwrap()
        else {
            panic!("a member elsewhere should create an organisation");
        };
        let moved: (uuid::Uuid, uuid::Uuid) =
            sqlx::query_as("select user_id, organisation_id from login_sessions where id = $1")
                .bind(session)
                .fetch_one(&mut *db)
                .await
                .unwrap();
        assert_eq!(moved, (user_id, created));
    }
}
