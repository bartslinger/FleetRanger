use std::net::IpAddr;

use sqlx::PgConnection;
use uuid::Uuid;

/// The account someone logged in with, as the login provider vouches for it.
pub struct Account<'a> {
    pub subject: &'a str,
    /// Only an email the provider has verified.
    pub email: Option<&'a str>,
    pub name: Option<&'a str>,
}

/// A user someone can log in as: theirs in one organisation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Membership {
    pub user_id: Uuid,
    pub organisation_id: Uuid,
}

/// The users a Google account logs in as, one per organisation (with
/// `System`): those that aren't archived, in organisations that aren't, the
/// one used last first. A first login in an organisation is matched on its
/// verified email, which ties that login to the account from then on.
pub async fn memberships(
    db: &mut PgConnection,
    subject: &str,
    verified_email: Option<&str>,
) -> sqlx::Result<Vec<Membership>> {
    if let Some(email) = verified_email {
        sqlx::query!(
            "update logins set oidc_subject = $1
             where email = lower($2) and oidc_subject is null
               and organisation_id not in (select organisation_id from logins
                                           where oidc_subject = $1)",
            subject,
            email,
        )
        .execute(&mut *db)
        .await?;
    }
    sqlx::query_as!(
        Membership,
        "select l.user_id, l.organisation_id
         from logins l join users u on u.id = l.user_id
         join organisations o on o.id = l.organisation_id
         where l.oidc_subject = $1 and u.archived_at is null and o.archived_at is null
         order by (select max(s.last_seen_at) from login_sessions s
                   where s.user_id = l.user_id) desc nulls last,
                  l.created_at",
        subject,
    )
    .fetch_all(&mut *db)
    .await
}

/// Starts a login session for the account (with `System`, as the login
/// comes before an organisation is known), in `membership`'s organisation
/// if given; otherwise the person chooses one first.
pub async fn create_session(
    db: &mut PgConnection,
    account: &Account<'_>,
    membership: Option<Membership>,
    token_sha256: &[u8],
    idle_days: i32,
    ip: IpAddr,
    user_agent: Option<&str>,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "insert into login_sessions (subject, email, name, organisation_id, user_id,
                                     token_sha256, expires_at, ip, user_agent)
         values ($1, lower($2), $3, $4, $5, $6, now() + make_interval(days => $7), $8, $9)
         returning id",
        account.subject,
        account.email,
        account.name,
        membership.map(|m| m.organisation_id),
        membership.map(|m| m.user_id),
        token_sha256,
        idle_days,
        ip as IpAddr,
        user_agent,
    )
    .fetch_one(&mut *db)
    .await
}

/// A valid login session: the person's, and their user once they are in an
/// organisation.
pub struct Session {
    pub id: Uuid,
    pub subject: String,
    /// As the login provider gave them at login.
    pub email: Option<String>,
    pub name: Option<String>,
    pub member: Option<Member>,
}

/// The user a login session is in an organisation as.
pub struct Member {
    pub user_id: Uuid,
    pub organisation_id: Uuid,
    /// The login's.
    pub email: String,
    pub name: String,
}

/// The valid session with this token hash, extending its expiry because it is
/// in use (with `System`: the session says which organisation the request is
/// in, if any). None if it is unknown, expired or revoked, or its user is
/// archived.
pub async fn use_session(
    db: &mut PgConnection,
    token_sha256: &[u8],
    idle_days: i32,
) -> sqlx::Result<Option<Session>> {
    let Some(row) = sqlx::query!(
        r#"with s as (
               update login_sessions
               set last_seen_at = now(), expires_at = now() + make_interval(days => $2)
               where token_sha256 = $1 and revoked_at is null and expires_at > now()
               returning id, subject, email, name, user_id, organisation_id)
           select s.id as "id!", s.subject as "subject!", s.email, s.name,
                  s.user_id, s.organisation_id,
                  l.email as "login_email?", u.name as "user_name?", u.archived_at
           from s
           left join logins l on l.user_id = s.user_id
           left join users u on u.id = s.user_id"#,
        token_sha256,
        idle_days,
    )
    .fetch_optional(&mut *db)
    .await?
    else {
        return Ok(None);
    };
    let member = match (row.user_id, row.organisation_id) {
        (Some(user_id), Some(organisation_id)) => {
            let (Some(email), Some(name), None) = (row.login_email, row.user_name, row.archived_at)
            else {
                return Ok(None);
            };
            Some(Member {
                user_id,
                organisation_id,
                email,
                name,
            })
        }
        _ => None,
    };
    Ok(Some(Session {
        id: row.id,
        subject: row.subject,
        email: row.email,
        name: row.name,
        member,
    }))
}

pub struct ImpersonatedUser {
    pub name: String,
    pub email: Option<String>,
}

/// A user an admin impersonates, unless archived: with or without a login.
pub async fn impersonated_user(
    db: &mut PgConnection,
    user_id: Uuid,
) -> sqlx::Result<Option<ImpersonatedUser>> {
    sqlx::query_as!(
        ImpersonatedUser,
        r#"select u.name, l.email as "email?" from users u
           left join logins l on l.user_id = u.id
           where u.id = $1 and u.archived_at is null"#,
        user_id,
    )
    .fetch_optional(&mut *db)
    .await
}

/// The user's roles: per group, and with no group on the whole organisation.
pub async fn roles(
    db: &mut PgConnection,
    user_id: Uuid,
) -> sqlx::Result<Vec<(Option<Uuid>, String)>> {
    let rows = sqlx::query!(
        r#"select group_id as "group_id?", role as "role!" from group_members where user_id = $1
           union all
           select null, role from organisation_roles where user_id = $1"#,
        user_id,
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.group_id, row.role))
        .collect())
}

/// Ends a login session (with `System`: also an expired one, whose request
/// has no organisation); the user it was in an organisation as, if it was
/// still open and in one.
pub async fn revoke_session(
    db: &mut PgConnection,
    token_sha256: &[u8],
) -> sqlx::Result<Option<Membership>> {
    let row = sqlx::query!(
        "update login_sessions set revoked_at = now()
         where token_sha256 = $1 and revoked_at is null
         returning user_id, organisation_id",
        token_sha256,
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(row.and_then(|row| {
        Some(Membership {
            user_id: row.user_id?,
            organisation_id: row.organisation_id?,
        })
    }))
}

/// Whether there is more than one group.
pub async fn several_groups(db: &mut PgConnection) -> sqlx::Result<bool> {
    sqlx::query_scalar!(r#"select count(*) > 1 as "several!" from groups"#)
        .fetch_one(&mut *db)
        .await
}

/// One of the organisations a person can open.
#[derive(serde::Serialize)]
pub struct OrganisationChoice {
    pub id: Uuid,
    pub name: String,
    /// The one the login session is in.
    pub current: bool,
    /// The one the person used last: the current one, or before choosing,
    /// the one of their most recently used other session.
    pub last_used: bool,
}

/// The organisations of the person a login session is for (with `System`):
/// those where their account has a login, as a user who isn't archived, in
/// an organisation that isn't archived; by name.
pub async fn organisations_of(
    db: &mut PgConnection,
    session_id: Uuid,
) -> sqlx::Result<Vec<OrganisationChoice>> {
    sqlx::query_as!(
        OrganisationChoice,
        r#"with last as (
               select coalesce(s.organisation_id,
                               (select other.organisation_id from login_sessions other
                                where other.subject = s.subject
                                  and other.organisation_id is not null
                                order by other.last_seen_at desc
                                limit 1)) as organisation_id
               from login_sessions s where s.id = $1)
           select o.id, o.name,
                  o.id is not distinct from s.organisation_id as "current!",
                  o.id is not distinct from last.organisation_id as "last_used!"
           from login_sessions s
           cross join last
           join logins l on l.oidc_subject = s.subject
           join users u on u.id = l.user_id and u.archived_at is null
           join organisations o on o.id = l.organisation_id and o.archived_at is null
           where s.id = $1
           order by lower(o.name), o.id"#,
        session_id,
    )
    .fetch_all(&mut *db)
    .await
}

/// Puts a login session in another of the person's organisations, as their
/// user there (with `System`). False if they have none there.
pub async fn switch_organisation(
    db: &mut PgConnection,
    session_id: Uuid,
    organisation_id: Uuid,
) -> sqlx::Result<bool> {
    let switched = sqlx::query!(
        "update login_sessions s set user_id = l.user_id, organisation_id = l.organisation_id
         from logins l, users u, organisations o
         where s.id = $1 and l.oidc_subject = s.subject and l.organisation_id = $2
           and u.id = l.user_id and u.archived_at is null
           and o.id = l.organisation_id and o.archived_at is null",
        session_id,
        organisation_id,
    )
    .execute(&mut *db)
    .await?;
    Ok(switched.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::{Account, Membership, create_session, memberships};
    use crate::testing;

    const EMAIL: &str = "test-person@example.org";

    /// Someone in two organisations has a login in each: their first
    /// sign-in ties both to their Google account, the one used last comes
    /// first, and another account with the same email takes over neither.
    /// A session starts without an organisation and is put in one only
    /// where the person has a user that isn't archived.
    #[sqlx::test]
    async fn one_account_logs_in_to_its_own_organisations(pool: PgPool) {
        let mut a = testing::db(&pool).await;
        let a_id = testing::organisation(&mut a).await;
        let in_a = testing::user(&mut a, "Test Person", &[]).await;
        let b_id = testing::second_organisation(&mut a, "Test Org B").await;
        let in_b: Uuid = sqlx::query_scalar(
            "insert into users (organisation_id, name) values ($1, 'Test Person') returning id",
        )
        .bind(b_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        for (organisation, user) in [(a_id, in_a), (b_id, in_b)] {
            sqlx::query("insert into logins (organisation_id, user_id, email) values ($1, $2, $3)")
                .bind(organisation)
                .bind(user)
                .bind(EMAIL)
                .execute(&pool)
                .await
                .unwrap();
        }
        let in_a = Membership {
            user_id: in_a,
            organisation_id: a_id,
        };
        let in_b = Membership {
            user_id: in_b,
            organisation_id: b_id,
        };

        // Logging in comes before an organisation is known.
        let mut system = pool.acquire().await.unwrap();
        let first = memberships(&mut system, "test-subject", Some("Test-Person@example.org"))
            .await
            .unwrap();
        assert_eq!(first.len(), 2, "both organisations' logins");

        let account = Account {
            subject: "test-subject",
            email: Some(EMAIL),
            name: Some("Test Person"),
        };
        create_session(
            &mut system,
            &account,
            Some(in_b),
            &[1; 32],
            30,
            [127, 0, 0, 1].into(),
            None,
        )
        .await
        .unwrap();
        let again = memberships(&mut system, "test-subject", None)
            .await
            .unwrap();
        assert_eq!(again, [in_b, in_a], "the one used last first");

        let other = memberships(&mut system, "other-subject", Some(EMAIL))
            .await
            .unwrap();
        assert_eq!(other, []);

        // A new session has no organisation until the person picks one;
        // the one used last is marked.
        let session = create_session(
            &mut system,
            &account,
            None,
            &[2; 32],
            30,
            [127, 0, 0, 1].into(),
            None,
        )
        .await
        .unwrap();
        let listed = |choices: Vec<super::OrganisationChoice>| -> Vec<(Uuid, bool, bool)> {
            choices
                .into_iter()
                .map(|o| (o.id, o.current, o.last_used))
                .collect()
        };
        let mut expected = vec![(a_id, false, false), (b_id, false, true)];
        expected.sort();
        let mut choices = listed(super::organisations_of(&mut system, session).await.unwrap());
        choices.sort();
        assert_eq!(choices, expected);

        assert!(
            super::switch_organisation(&mut system, session, a_id)
                .await
                .unwrap()
        );
        let (user, organisation): (Uuid, Uuid) =
            sqlx::query_as("select user_id, organisation_id from login_sessions where id = $1")
                .bind(session)
                .fetch_one(&mut *system)
                .await
                .unwrap();
        assert_eq!((user, organisation), (in_a.user_id, a_id));
        let elsewhere = testing::second_organisation(&mut a, "Test Org C").await;
        assert!(
            !super::switch_organisation(&mut system, session, elsewhere)
                .await
                .unwrap()
        );
        sqlx::query("update users set archived_at = now() where id = $1")
            .bind(in_b.user_id)
            .execute(&mut *system)
            .await
            .unwrap();
        assert!(
            !super::switch_organisation(&mut system, session, b_id)
                .await
                .unwrap()
        );
        let choices = super::organisations_of(&mut system, session).await.unwrap();
        assert_eq!(listed(choices), [(a_id, true, true)]);
    }

    /// The database refuses a session whose user and account aren't one
    /// login, so the two can't drift apart.
    #[sqlx::test]
    async fn a_session_matches_its_login(pool: PgPool) {
        let mut db = testing::db(&pool).await;
        let organisation_id = testing::organisation(&mut db).await;
        let user_id = testing::user(&mut db, "Test Person", &[]).await;
        sqlx::query("insert into logins (user_id, email, oidc_subject) values ($1, $2, 'mine')")
            .bind(user_id)
            .bind(EMAIL)
            .execute(&mut *db)
            .await
            .unwrap();
        let mut system = pool.acquire().await.unwrap();
        let account = |subject| Account {
            subject,
            email: Some(EMAIL),
            name: None,
        };
        let membership = Some(Membership {
            user_id,
            organisation_id,
        });
        let ip = [127, 0, 0, 1].into();
        create_session(
            &mut system,
            &account("mine"),
            membership,
            &[1; 32],
            30,
            ip,
            None,
        )
        .await
        .unwrap();
        let someone_else = create_session(
            &mut system,
            &account("theirs"),
            membership,
            &[2; 32],
            30,
            ip,
            None,
        )
        .await;
        assert!(someone_else.is_err());
        let changed = sqlx::query("update logins set oidc_subject = null where user_id = $1")
            .bind(user_id)
            .execute(&mut *system)
            .await;
        assert!(changed.is_err(), "the session still points at the account");

        // An admin changing the email logs the old account out first.
        let fields = crate::users::db::UserFields {
            name: "Test Person",
            roles: &[],
            email: Some("test-new@example.org"),
        };
        let updated = crate::users::db::update(&mut db, user_id, &fields).await;
        assert!(matches!(updated, Ok(Ok(()))));
        let sessions: i64 = sqlx::query_scalar("select count(*) from login_sessions")
            .fetch_one(&mut *system)
            .await
            .unwrap();
        assert_eq!(sessions, 0);
    }
}
