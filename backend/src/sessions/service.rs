use serde::Serialize;
use sqlx::PgPool;
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
    db: &PgPool,
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
    db: &PgPool,
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
/// - `group`: the session's group, None for the organisation; if not given,
///   the flights' group when they share one, else the organisation.
pub async fn create(
    db: &PgPool,
    actor: &Actor,
    flight_ids: &[Uuid],
    group: Option<Option<Uuid>>,
) -> ApiResult<Uuid> {
    let flight_ids = seen(db, actor, flight_ids).await?;
    let group = match group {
        Some(group) => group,
        None => {
            let flight_groups = flights::db::groups_of(db, &flight_ids)
                .await?
                .ok_or(ApiError::NotFound)?;
            let first = flight_groups[0];
            first.filter(|_| flight_groups.iter().all(|group| *group == first))
        }
    };
    let mut changed = db::groups_left(db, &flight_ids).await?;
    changed.push(group);
    policy::change_sessions(&actor.grants, &changed)?;
    if let Some(group) = group
        && !flights::db::group_exists(db, group).await?
    {
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
async fn seen(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Vec<Uuid>> {
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
async fn not_archived(db: &PgPool, flight_ids: &[Uuid]) -> ApiResult<()> {
    if flights::db::any_archived(db, flight_ids).await? {
        return Err(ApiError::Invalid(
            "Archived flights can’t be in a session. Restore them first.".to_owned(),
        ));
    }
    Ok(())
}

/// Adds these flights to a session. They leave the sessions they were in.
pub async fn add(db: &PgPool, actor: &Actor, session: Uuid, flight_ids: &[Uuid]) -> ApiResult<()> {
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
pub async fn remove(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<u64> {
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
pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<Session> {
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
pub async fn delete(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    let groups = db::groups_of(db, &[id]).await?.ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    if !db::delete(db, id).await? {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

/// Merges sessions into the one that starts first, and returns it.
pub async fn merge(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<Uuid> {
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
pub async fn split(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
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

/// Moves a session to a group, or to the organisation with None, and with
/// `with_flights` its flights as well.
pub async fn set_group(
    db: &PgPool,
    actor: &Actor,
    id: Uuid,
    group: Option<Uuid>,
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
    if let Some(group) = group
        && !flights::db::group_exists(db, group).await?
    {
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

    async fn actor(db: &PgPool, user_id: Uuid) -> Actor {
        Actor {
            user_id,
            grants: crate::auth::grants(db, user_id).await.unwrap(),
            login_session_id: Uuid::nil(),
            ip: [127, 0, 0, 1].into(),
            email: None,
            impersonated_by: None,
            name: String::new(),
        }
    }

    async fn flight(db: &PgPool, group: Uuid) -> Uuid {
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
        .fetch_one(db)
        .await
        .unwrap()
    }

    /// A change needs `sessions.edit` on every session that changes: an
    /// editor of North can't take a flight out of South's session, also not
    /// a North flight; a session made by hand gets the flights' group, or
    /// the organisation when they have several.
    #[sqlx::test]
    async fn changes_need_every_session_that_changes(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let south = testing::group(&db, "South").await;
        let editor: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        testing::join(&db, editor, north).await;
        let editor = actor(&db, editor).await;
        let admin = Actor {
            grants: Grants::from_stored([(None, "admin".to_owned())]).unwrap(),
            ..editor.clone()
        };

        let (ours, theirs, also_ours) = (
            flight(&db, north).await,
            flight(&db, south).await,
            flight(&db, north).await,
        );
        let made = async |actor: &Actor, flights: &[Uuid], group| {
            create(&db, actor, flights, group)
                .await
                .ok()
                .expect("a session")
        };
        let south_session = made(&admin, &[ours], Some(Some(south))).await;
        // Leaving South's session, joining a new North one.
        let refused = create(&db, &editor, &[ours], None).await;
        assert!(matches!(refused, Err(ApiError::Conflict(_))));
        assert!(remove(&db, &editor, &[ours]).await.is_err());

        let own = made(&editor, &[also_ours], None).await;
        assert_eq!(
            db::groups_of(&db, &[own]).await.unwrap(),
            Some(vec![Some(north)])
        );
        assert!(
            add(&db, &editor, south_session, &[also_ours])
                .await
                .is_err()
        );

        let joint = made(&admin, &[ours, theirs], None).await;
        assert_eq!(
            db::groups_of(&db, &[joint]).await.unwrap(),
            Some(vec![None])
        );
    }

    /// A session with comments keeps its last flight until it is deleted
    /// explicitly, and merging keeps its comments; one without comments goes
    /// when its last flight leaves.
    #[sqlx::test]
    async fn comments_keep_their_session(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let editor = testing::user(&db, "A", &["editor"]).await;
        let editor = testing::actor(&db, editor).await;
        let (a, b, c) = (
            flight(&db, north).await,
            flight(&db, north).await,
            flight(&db, north).await,
        );
        let commented = create(&db, &editor, &[a], None).await.ok().unwrap();
        let other = create(&db, &editor, &[b], None).await.ok().unwrap();
        comments::add(&db, &editor, commented, "Landed early: wind")
            .await
            .ok()
            .unwrap();

        assert!(matches!(
            remove(&db, &editor, &[a]).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            add(&db, &editor, other, &[a]).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(create(&db, &editor, &[a], None).await.is_err());
        assert_eq!(db::session_of(&db, &[a]).await.unwrap(), Some(commented));

        let merged = merge(&db, &editor, &[commented, other]).await.ok().unwrap();
        let gone = if merged == commented {
            other
        } else {
            commented
        };
        assert_eq!(db::groups_of(&db, &[gone]).await.unwrap(), None);
        assert_eq!(comments::of_session(&db, merged).await.unwrap().len(), 1);

        delete(&db, &editor, merged).await.ok().unwrap();
        assert_eq!(db::groups_of(&db, &[merged]).await.unwrap(), None);
        assert_eq!(db::session_of(&db, &[a, b]).await.unwrap(), None);

        let plain = create(&db, &editor, &[c], None).await.ok().unwrap();
        remove(&db, &editor, &[c]).await.ok().unwrap();
        assert_eq!(db::groups_of(&db, &[plain]).await.unwrap(), None);
    }

    /// Comments are written with `sessions.edit`, and only their authors
    /// change them; unchanged text isn't marked edited.
    #[sqlx::test]
    async fn only_authors_change_comments(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let author = testing::user(&db, "A", &["editor"]).await;
        let author = testing::actor(&db, author).await;
        let other = testing::user(&db, "B", &["editor"]).await;
        let other = testing::actor(&db, other).await;
        let viewer = testing::user(&db, "C", &["viewer"]).await;
        let viewer = testing::actor(&db, viewer).await;
        let session = create(&db, &author, &[flight(&db, north).await], None)
            .await
            .ok()
            .unwrap();

        assert!(comments::add(&db, &viewer, session, "Seen").await.is_err());
        let id = comments::add(&db, &author, session, "Battery swap at 10:40")
            .await
            .ok()
            .unwrap();
        let edited = async || comments::of_session(&db, session).await.unwrap()[0].edited_at;
        assert!(
            comments::edit(&db, &other, session, id, "Changed")
                .await
                .is_err()
        );
        assert!(comments::delete(&db, &other, session, id).await.is_err());
        comments::edit(&db, &author, session, id, " Battery swap at 10:40 ")
            .await
            .ok()
            .unwrap();
        assert_eq!(edited().await, None);
        comments::edit(&db, &author, session, id, "Battery swap at 10:45")
            .await
            .ok()
            .unwrap();
        assert!(edited().await.is_some());
        comments::delete(&db, &author, session, id)
            .await
            .ok()
            .unwrap();
        assert!(comments::of_session(&db, session).await.unwrap().is_empty());
    }
}
