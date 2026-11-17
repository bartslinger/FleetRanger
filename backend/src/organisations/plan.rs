//! An organisation's plan and its limits (user's decision, 2026-11-17: a plan
//! per organisation, the limits per plan in the code). Sign-up makes free
//! organisations; the rest are unlimited. The limits are checked only when a
//! login or controller is added, never by removing existing ones.

use sqlx::PgConnection;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Plan {
    Free,
    Unlimited,
}

/// What a plan allows that would be more than it does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Limit {
    Logins(i64),
    Controllers(i64),
}

impl Limit {
    /// What an admin, or the controller's pilot, is told.
    pub fn message(self) -> String {
        match self {
            Self::Logins(max) => {
                format!("The free plan has {max} logins. Remove a login to invite someone else.")
            }
            Self::Controllers(max) => format!(
                "The free plan has {max} controllers. Unpair one in FleetRanger to pair this one."
            ),
        }
    }
}

impl Plan {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "free" => Some(Self::Free),
            "unlimited" => Some(Self::Unlimited),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Unlimited => "unlimited",
        }
    }

    /// Logins, invitations not yet accepted included.
    fn max_logins(self) -> Option<i64> {
        match self {
            Self::Free => Some(2),
            Self::Unlimited => None,
        }
    }

    /// Paired controllers.
    fn max_controllers(self) -> Option<i64> {
        match self {
            Self::Free => Some(2),
            Self::Unlimited => None,
        }
    }
}

/// The plan of the connection's organisation, locked until the transaction
/// ends, so two additions can't both take the last place.
async fn lock(db: &mut PgConnection) -> sqlx::Result<Plan> {
    sqlx::query!(
        "select pg_advisory_xact_lock(hashtext('fleetranger.plan.' || current_setting('app.organisation_id', true)))"
    )
    .execute(&mut *db)
    .await?;
    let plan = sqlx::query_scalar!(
        "select plan from organisations
         where id = current_setting('app.organisation_id', true)::uuid"
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(Plan::parse(&plan).unwrap_or(Plan::Free))
}

/// Whether one more login fits, in the caller's transaction.
pub async fn check_login(db: &mut PgConnection) -> sqlx::Result<Result<(), Limit>> {
    let Some(max) = lock(db).await?.max_logins() else {
        return Ok(Ok(()));
    };
    let count = sqlx::query_scalar!(
        r#"select count(*) as "count!" from logins
           where organisation_id = current_setting('app.organisation_id', true)::uuid"#
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(if count < max {
        Ok(())
    } else {
        Err(Limit::Logins(max))
    })
}

/// Whether the controller with this serial can pair, in the caller's
/// transaction: pairing again takes no new place.
pub async fn check_controller(
    db: &mut PgConnection,
    serial: &str,
) -> sqlx::Result<Result<(), Limit>> {
    let Some(max) = lock(db).await?.max_controllers() else {
        return Ok(Ok(()));
    };
    let count = sqlx::query_scalar!(
        r#"select count(*) as "count!" from remote_controllers
           where organisation_id = current_setting('app.organisation_id', true)::uuid
             and token_sha256 is not null and serial <> $1"#,
        serial,
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(if count < max {
        Ok(())
    } else {
        Err(Limit::Controllers(max))
    })
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::Limit;
    use crate::pairing::db::{add_code, unpair, use_code};
    use crate::users::db::{UserFields, insert, update};

    fn login(email: &str) -> UserFields<'_> {
        UserFields {
            name: "Test user",
            roles: &[],
            email: Some(email),
            invited_by: None,
        }
    }

    /// A free organisation takes no third login or controller; a changed
    /// address, a controller pairing again and one unpaired take no place.
    #[sqlx::test]
    async fn the_free_plan_limits_new_logins_and_controllers(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let organisation = crate::testing::organisation(&mut db).await;
        sqlx::query("update organisations set plan = 'free' where id = $1")
            .bind(organisation)
            .execute(&pool)
            .await
            .unwrap();

        let first = insert(&mut db, &login("test-a@example.org")).await.unwrap();
        assert!(first.is_ok());
        assert!(
            insert(&mut db, &login("test-b@example.org"))
                .await
                .unwrap()
                .is_ok()
        );
        assert_eq!(
            insert(&mut db, &login("test-c@example.org"))
                .await
                .unwrap()
                .err(),
            Some(Limit::Logins(2))
        );
        let first = first.unwrap_or_default();
        assert!(
            update(&mut db, first, &login("test-a2@example.org"))
                .await
                .unwrap()
                .is_ok()
        );

        let group = crate::testing::general(&mut db).await;
        let soon = Utc::now() + Duration::hours(1);
        for code in ["AAAA2222", "BBBB2222", "CCCC2222", "DDDD2222"] {
            add_code(&mut db, code, first, group, soon).await.unwrap();
        }
        let pair = async |db: &mut crate::db::Db, code, serial: &str| {
            use_code(db, code, serial, serial.as_bytes())
                .await
                .unwrap()
                .map(|paired| paired.err())
        };
        assert_eq!(pair(&mut db, "AAAA2222", "RC1").await, Some(None));
        assert_eq!(pair(&mut db, "BBBB2222", "RC2").await, Some(None));
        assert_eq!(
            pair(&mut db, "CCCC2222", "RC3").await,
            Some(Some(Limit::Controllers(2)))
        );
        // Refused, the code is still there.
        assert_eq!(pair(&mut db, "CCCC2222", "RC1").await, Some(None));
        let rc2: uuid::Uuid =
            sqlx::query_scalar("select id from remote_controllers where serial = 'RC2'")
                .fetch_one(&mut *db)
                .await
                .unwrap();
        assert!(unpair(&mut db, rc2).await.unwrap());
        assert_eq!(pair(&mut db, "DDDD2222", "RC3").await, Some(None));
    }
}
