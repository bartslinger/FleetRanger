use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::comments::{self, Comment};
use super::db::{self, SessionSummary, Window};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{
        self,
        db::{Filter, FlightDetail},
    },
    policy::{self, Permission},
};

/// One page of the sessions the actor may see that pass the filter, if given
/// only those with a flight in the window, and how many there are in all.
pub async fn list(
    db: &mut PgConnection,
    actor: &Actor,
    filter: &Filter,
    window: Option<Window>,
    limit: i64,
    offset: i64,
    with_tracks: bool,
) -> ApiResult<(Vec<SessionSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let mut sessions = db::list(db, &scope, filter, window, limit, offset, with_tracks).await?;
    if with_tracks {
        for session in &mut sessions {
            simplify_tracks(&mut session.flight_times);
        }
    }
    let total = db::count(db, &scope, filter, window).await?;
    Ok((sessions, total))
}

/// About 5 m, in degrees: finer than a map of a week's or month's sessions
/// can show.
const PREVIEW_TOLERANCE: f64 = 0.00005;

/// Each flight's `track` (its coordinates) as a coarse [lon, lat] line, for
/// a map of many sessions at once: a few dozen points instead of thousands.
fn simplify_tracks(flights: &mut serde_json::Value) {
    use geo::{LineString, Simplify};
    let Some(flights) = flights.as_array_mut() else {
        return;
    };
    for flight in flights {
        let Some(track) = flight.get_mut("track") else {
            continue;
        };
        let points: Vec<(f64, f64)> = track
            .as_array()
            .map(|points| {
                points
                    .iter()
                    .filter_map(|point| Some((point.get(0)?.as_f64()?, point.get(1)?.as_f64()?)))
                    .collect()
            })
            .unwrap_or_default();
        let line = LineString::from(points).simplify(PREVIEW_TOLERANCE);
        // Six decimals, about 10 cm: the rest is only bytes.
        let round = |value: f64| (value * 1e6).round() / 1e6;
        *track = line
            .points()
            .map(|point| serde_json::json!([round(point.x()), round(point.y())]))
            .collect();
    }
}

/// A session's place in the list of those the actor may see that pass the
/// filter. Not found if it isn't in that list.
pub async fn neighbours(
    db: &mut PgConnection,
    actor: &Actor,
    filter: &Filter,
    id: Uuid,
) -> ApiResult<db::Neighbours> {
    let scope = policy::flight_scope(actor);
    db::neighbours(db, &scope, filter, id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Puts these flights into a new session, and returns it. They leave the
/// sessions they were in.
///
/// - `group`: the session's group; if not given, the flights' group when
///   they share one.
pub async fn create(
    db: &mut PgConnection,
    actor: &Actor,
    flight_ids: &[Uuid],
    group: Option<Uuid>,
) -> ApiResult<Uuid> {
    let flight_ids = seen(db, actor, flight_ids).await?;
    let group = match group {
        Some(group) => group,
        None => {
            let flight_groups = flights::db::groups_of(db, &flight_ids)
                .await?
                .ok_or(ApiError::NotFound)?;
            let first = flight_groups[0];
            if flight_groups.iter().any(|group| *group != first) {
                return Err(ApiError::Invalid(
                    "These flights are in several groups. Choose the session's group.".to_owned(),
                ));
            }
            first
        }
    };
    let mut changed = db::groups_left(db, &flight_ids).await?;
    changed.push(group);
    policy::change_sessions(&actor.grants, &changed)?;
    if !flights::db::group_exists(db, group).await? {
        return Err(ApiError::Invalid("Choose an existing group.".to_owned()));
    }
    not_archived(db, &flight_ids).await?;
    db::split(db, &flight_ids, group)
        .await?
        .ok_or_else(stranded)
}

/// These flights, once, if there are any and the actor sees them all: only
/// flights someone sees go into a session. Flights they don't see are not
/// found, so their existence isn't revealed.
async fn seen(db: &mut PgConnection, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Vec<Uuid>> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    if !flights::db::all_visible(db, &policy::flight_scope(actor), &flight_ids).await? {
        return Err(ApiError::NotFound);
    }
    Ok(flight_ids)
}

/// Archived flights are never in a session.
async fn not_archived(db: &mut PgConnection, flight_ids: &[Uuid]) -> ApiResult<()> {
    if flights::db::any_archived(db, flight_ids).await? {
        return Err(ApiError::Invalid(
            "Archived flights can’t be in a session. Restore them first.".to_owned(),
        ));
    }
    Ok(())
}

/// Adds these flights to a session. They leave the sessions they were in.
pub async fn add(
    db: &mut PgConnection,
    actor: &Actor,
    session: Uuid,
    flight_ids: &[Uuid],
) -> ApiResult<()> {
    let flight_ids = seen(db, actor, flight_ids).await?;
    let mut changed = db::groups_of(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    changed.extend(db::groups_left(db, &flight_ids).await?);
    policy::change_sessions(&actor.grants, &changed)?;
    not_archived(db, &flight_ids).await?;
    db::add(db, session, &flight_ids)
        .await?
        .ok_or_else(stranded)
}

/// Takes these flights out of their sessions; they stay flights, without a
/// session. A session left without flights is deleted. Returns how many were
/// in a session.
///
/// Here and when flights move to another session, a session with comments
/// keeps its last flight: it is deleted only explicitly.
pub async fn remove(db: &mut PgConnection, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<u64> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    if flights::db::groups_of(db, &flight_ids).await?.is_none() {
        return Err(ApiError::NotFound);
    }
    let changed = db::groups_left(db, &flight_ids).await?;
    policy::change_sessions(&actor.grants, &changed)?;
    db::remove(db, &flight_ids).await?.ok_or_else(stranded)
}

fn stranded() -> ApiError {
    ApiError::Conflict(
        "A session with comments can’t lose its last flight. Delete the session instead."
            .to_owned(),
    )
}

/// A session with the flights of it the actor may see.
#[derive(Serialize)]
pub struct Session {
    pub id: Uuid,
    #[serde(flatten)]
    pub group: db::OwnGroup,
    /// In takeoff order.
    pub flights: Vec<FlightDetail>,
    /// Oldest first.
    pub comments: Vec<Comment>,
}

/// A session the actor may see a flight of. Any other session is not found,
/// so its existence isn't revealed.
pub async fn get(db: &mut PgConnection, actor: &Actor, id: Uuid) -> ApiResult<Session> {
    let scope = policy::flight_scope(actor);
    let mut flights = flights::db::of_session(db, &scope, id).await?;
    if flights.is_empty() {
        return Err(ApiError::NotFound);
    }
    let group = db::own_group(db, id).await?.ok_or(ApiError::NotFound)?;
    flights::service::link_equipment(actor, &mut flights);
    let comments = comments::of_session(db, id).await?;
    Ok(Session {
        id,
        group,
        flights,
        comments,
    })
}

/// Deletes a session and its comments; its flights stay, without a session.
pub async fn delete(db: &mut PgConnection, actor: &Actor, id: Uuid) -> ApiResult<()> {
    let groups = db::groups_of(db, &[id]).await?.ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    if !db::delete(db, id).await? {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

/// Merges sessions into the one that starts first, and returns it.
pub async fn merge(db: &mut PgConnection, actor: &Actor, ids: &[Uuid]) -> ApiResult<Uuid> {
    let ids = distinct(ids);
    if ids.len() < 2 {
        return Err(ApiError::Invalid("Select two or more sessions.".to_owned()));
    }
    let groups = db::groups_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    Ok(db::merge(db, &ids).await?)
}

/// Moves flights of one session into a new session of the same group, and
/// returns it.
pub async fn split(db: &mut PgConnection, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
    let flight_ids = distinct(flight_ids);
    let session = db::session_of(db, &flight_ids)
        .await?
        .ok_or_else(|| ApiError::Invalid("Select flights of one session.".to_owned()))?;
    let groups = db::groups_of(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    let flights = db::flight_groups(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    if flight_ids.len() == flights.len() {
        return Err(ApiError::Invalid(
            "Leave at least one flight in the session.".to_owned(),
        ));
    }
    db::split(db, &flight_ids, groups[0])
        .await?
        .ok_or_else(stranded)
}

/// Moves a session to a group, and with `with_flights` its flights as well.
pub async fn set_group(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    group: Uuid,
    with_flights: bool,
) -> ApiResult<()> {
    // Refused before not found, so this doesn't tell whether sessions exist.
    if !actor.grants.anywhere(Permission::SessionsMove) {
        return Err(ApiError::Forbidden);
    }
    let from = db::groups_of(db, &[id]).await?.ok_or(ApiError::NotFound)?[0];
    let flight_groups = if with_flights {
        Some(
            db::flight_groups(db, &[id])
                .await?
                .ok_or(ApiError::NotFound)?,
        )
    } else {
        None
    };
    policy::move_session(&actor.grants, from, group, flight_groups.as_deref())?;
    if !flights::db::group_exists(db, group).await? {
        return Err(ApiError::Invalid("Choose an existing group.".to_owned()));
    }
    if !db::set_group(db, id, group, with_flights).await? {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

fn distinct(ids: &[Uuid]) -> Vec<Uuid> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{policy::Grants, testing};

    async fn actor(db: &mut PgConnection, user_id: Uuid) -> Actor {
        Actor {
            user_id,
            organisation_id: crate::testing::organisation(db).await,
            grants: crate::auth::grants(db, user_id).await.unwrap(),
            login_session_id: Uuid::nil(),
            ip: [127, 0, 0, 1].into(),
            email: None,
            impersonated_by: None,
            name: String::new(),
        }
    }

    async fn flight(db: &mut PgConnection, group: Uuid) -> Uuid {
        sqlx::query_scalar(
            "with log as (
                 insert into raw_logs (sha256, path, size_bytes)
                 values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
                 returning id)
             insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, group_id)
             select id, now(), now(), 0, 0, '{}', $1 from log
             returning id",
        )
        .bind(group)
        .fetch_one(&mut *db)
        .await
        .unwrap()
    }

    /// A change needs `sessions.edit` on every session that changes: an
    /// editor of North can't take a flight out of South's session, also not
    /// a North flight; a session made by hand gets the flights' group, and
    /// when they have several one must be chosen.
    #[sqlx::test]
    async fn changes_need_every_session_that_changes(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let north = testing::group(&mut db, "North").await;
        let south = testing::group(&mut db, "South").await;
        let editor: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&mut *db)
            .await
            .unwrap();
        testing::join(&mut db, editor, north).await;
        let editor = actor(&mut db, editor).await;
        let admin = Actor {
            grants: Grants::from_stored([(None, "admin".to_owned())]).unwrap(),
            ..editor.clone()
        };

        let (ours, theirs, also_ours) = (
            flight(&mut db, north).await,
            flight(&mut db, south).await,
            flight(&mut db, north).await,
        );
        let made = async |db: &mut PgConnection, actor: &Actor, flights: &[Uuid], group| {
            create(db, actor, flights, group)
                .await
                .ok()
                .expect("a session")
        };
        let south_session = made(&mut db, &admin, &[ours], Some(south)).await;
        // Leaving South's session, joining a new North one.
        let refused = create(&mut db, &editor, &[ours], None).await;
        assert!(matches!(refused, Err(ApiError::Conflict(_))));
        assert!(remove(&mut db, &editor, &[ours]).await.is_err());

        let own = made(&mut db, &editor, &[also_ours], None).await;
        assert_eq!(
            db::groups_of(&mut db, &[own]).await.unwrap(),
            Some(vec![north])
        );
        assert!(
            add(&mut db, &editor, south_session, &[also_ours])
                .await
                .is_err()
        );

        assert!(matches!(
            create(&mut db, &admin, &[ours, theirs], None).await,
            Err(ApiError::Invalid(_))
        ));
        let joint = made(&mut db, &admin, &[ours, theirs], Some(south)).await;
        assert_eq!(
            db::groups_of(&mut db, &[joint]).await.unwrap(),
            Some(vec![south])
        );
    }

    /// A session with comments keeps its last flight until it is deleted
    /// explicitly, and merging keeps its comments; one without comments goes
    /// when its last flight leaves.
    #[sqlx::test]
    async fn comments_keep_their_session(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let north = testing::group(&mut db, "North").await;
        let editor = testing::member(&mut db, "A", north, &["editor"]).await;
        let editor = testing::actor(&mut db, editor).await;
        let (a, b, c) = (
            flight(&mut db, north).await,
            flight(&mut db, north).await,
            flight(&mut db, north).await,
        );
        let commented = create(&mut db, &editor, &[a], None).await.ok().unwrap();
        let other = create(&mut db, &editor, &[b], None).await.ok().unwrap();
        comments::add(&mut db, &editor, commented, "Landed early: wind")
            .await
            .ok()
            .unwrap();

        assert!(matches!(
            remove(&mut db, &editor, &[a]).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            add(&mut db, &editor, other, &[a]).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(create(&mut db, &editor, &[a], None).await.is_err());
        assert_eq!(
            db::session_of(&mut db, &[a]).await.unwrap(),
            Some(commented)
        );

        let merged = merge(&mut db, &editor, &[commented, other])
            .await
            .ok()
            .unwrap();
        let gone = if merged == commented {
            other
        } else {
            commented
        };
        assert_eq!(db::groups_of(&mut db, &[gone]).await.unwrap(), None);
        assert_eq!(
            comments::of_session(&mut db, merged).await.unwrap().len(),
            1
        );

        delete(&mut db, &editor, merged).await.ok().unwrap();
        assert_eq!(db::groups_of(&mut db, &[merged]).await.unwrap(), None);
        assert_eq!(db::session_of(&mut db, &[a, b]).await.unwrap(), None);

        let plain = create(&mut db, &editor, &[c], None).await.ok().unwrap();
        remove(&mut db, &editor, &[c]).await.ok().unwrap();
        assert_eq!(db::groups_of(&mut db, &[plain]).await.unwrap(), None);
    }

    /// Comments are written with `sessions.edit`, and only their authors
    /// change them; unchanged text isn't marked edited.
    #[sqlx::test]
    async fn only_authors_change_comments(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let north = testing::group(&mut db, "North").await;
        let author = testing::member(&mut db, "A", north, &["editor"]).await;
        let author = testing::actor(&mut db, author).await;
        let other = testing::member(&mut db, "B", north, &["editor"]).await;
        let other = testing::actor(&mut db, other).await;
        let viewer = testing::member(&mut db, "C", north, &["viewer"]).await;
        let viewer = testing::actor(&mut db, viewer).await;
        let flown = flight(&mut db, north).await;
        let session = create(&mut db, &author, &[flown], None).await.ok().unwrap();

        assert!(
            comments::add(&mut db, &viewer, session, "Seen")
                .await
                .is_err()
        );
        let id = comments::add(&mut db, &author, session, "Battery swap at 10:40")
            .await
            .ok()
            .unwrap();
        let edited = async |db: &mut PgConnection| {
            comments::of_session(db, session).await.unwrap()[0].edited_at
        };
        assert!(
            comments::edit(&mut db, &other, session, id, "Changed")
                .await
                .is_err()
        );
        assert!(
            comments::delete(&mut db, &other, session, id)
                .await
                .is_err()
        );
        comments::edit(&mut db, &author, session, id, " Battery swap at 10:40 ")
            .await
            .ok()
            .unwrap();
        assert_eq!(edited(&mut db).await, None);
        comments::edit(&mut db, &author, session, id, "Battery swap at 10:45")
            .await
            .ok()
            .unwrap();
        assert!(edited(&mut db).await.is_some());
        comments::delete(&mut db, &author, session, id)
            .await
            .ok()
            .unwrap();
        assert!(
            comments::of_session(&mut db, session)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
