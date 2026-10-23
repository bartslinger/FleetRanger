//! Migration 0018 fills groups and roles from teams: nobody's access may
//! change. Each check compares today's rule (see `policy`) with the same
//! permission under "Groups and roles" in docs/architecture.md.

use sqlx::{PgPool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!();

// Teams North and South. A in North, B in both, C the all-flights role,
// D admin, E in no team, F a former North pilot.
const BEFORE: &str = r#"
insert into teams (name) values ('North'), ('South');
insert into users (name, role) values
    ('A', 'member'), ('B', 'member'), ('C', 'all_flights'),
    ('D', 'admin'), ('E', 'member'), ('F', 'member');
insert into team_members (user_id, team_id)
select u.id, t.id from users u, teams t
where (u.name, t.name) in (('A', 'North'), ('B', 'North'), ('B', 'South'));

insert into raw_logs (sha256, path, size_bytes)
select sha256(n::text::bytea), n::text, 0 from generate_series(1, 8) n;
insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m, track,
                     team_id, pilot_id)
select r.id, now(), now(), 0, 0, '{}',
       (select id from teams where name = f.team), (select id from users where name = f.pilot)
from (values ('1', 'North', 'A'), ('2', 'North', 'F'), ('3', 'South', 'B'),
             ('4', null, null), ('5', null, 'E'), ('6', 'North', 'B'),
             ('7', 'South', 'B'), ('8', 'North', 'A')) as f (path, team, pilot)
join raw_logs r on r.path = f.path;

-- One team; mixed teams; a team and the queue.
insert into sessions (id) values
    ('00000000-0000-0000-0000-000000000001'),
    ('00000000-0000-0000-0000-000000000002'),
    ('00000000-0000-0000-0000-000000000003');
update flights f set session_id = s.id::uuid
from (values ('1', '00000000-0000-0000-0000-000000000001'),
             ('2', '00000000-0000-0000-0000-000000000001'),
             ('6', '00000000-0000-0000-0000-000000000002'),
             ('7', '00000000-0000-0000-0000-000000000002'),
             ('8', '00000000-0000-0000-0000-000000000003'),
             ('5', '00000000-0000-0000-0000-000000000003')) as s (path, id)
join raw_logs r on r.path = s.path
where f.raw_log_id = r.id;

insert into aircraft (serial, team_id) values
    ('TEST-DRONE-N', (select id from teams where name = 'North')),
    ('TEST-DRONE-0', null);
insert into remote_controllers (serial, team_id) values
    ('TEST-RC-S', (select id from teams where name = 'South'));
"#;

/// Holds a permission that Viewer and Editor include (reading), or only
/// Editor (changing), on the item's group or the whole organisation.
const READS: &str = "array['viewer', 'editor', 'admin']";
const CHANGES: &str = "array['editor', 'admin']";

fn holds(roles: &str, group: &str) -> String {
    format!(
        "(exists (select 1 from organisation_roles o where o.user_id = u.id and o.role = any({roles}))
          or exists (select 1 from group_members m where m.user_id = u.id and m.group_id = {group}
                     and m.role = any({roles})))"
    )
}

/// The users and items for which `before` and `after` disagree.
async fn differ(db: &PgPool, items: &str, before: &str, after: &str) -> Vec<String> {
    let sql = format!(
        "select u.name || ' ' || i.label from users u cross join ({items}) i
         where coalesce(({before}), false) <> coalesce(({after}), false) order by 1"
    );
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql)).fetch_all(db).await.unwrap()
}

const OWN_TEAM: &str = "i.team_id in (select team_id from team_members where user_id = u.id)";

#[sqlx::test(migrations = false)]
async fn fill_from_teams_keeps_everyones_access(db: PgPool) {
    MIGRATOR.run_to(17, &db).await.unwrap();
    sqlx::raw_sql(BEFORE).execute(&db).await.unwrap();
    MIGRATOR.run_to(18, &db).await.unwrap();

    let flights = "select 'flight ' || r.path as label, f.team_id, f.group_id, f.pilot_id
                   from flights f join raw_logs r on r.id = f.raw_log_id";

    // Seeing a flight: its pilot, its team, or all flights.
    let before = format!("u.role in ('all_flights', 'admin') or i.pilot_id = u.id or {OWN_TEAM}");
    let after = format!("i.pilot_id = u.id or {}", holds(READS, "i.group_id"));
    assert_eq!(differ(&db, flights, &before, &after).await, Vec::<String>::new());

    // Changing a flight (pilot, archiving): its team's members and admins.
    let before = format!("u.role = 'admin' or (u.role = 'member' and {OWN_TEAM})");
    let after = holds(CHANGES, "i.group_id");
    assert_eq!(differ(&db, flights, &before, &after).await, Vec::<String>::new());

    // Changing a session: members of all its flights' teams, and admins.
    // A session of mixed teams goes to the organisation, so it's left out
    // here and checked below.
    let sessions = "select 'session ' || right(s.id::text, 1) as label, s.id, s.group_id
                    from sessions s
                    where s.id <> '00000000-0000-0000-0000-000000000002'";
    let before = "u.role = 'admin' or (u.role = 'member' and not exists (
                      select 1 from flights f where f.session_id = i.id and (f.team_id is null
                          or f.team_id not in (select team_id from team_members where user_id = u.id))))";
    let after = holds(CHANGES, "i.group_id");
    assert_eq!(differ(&db, sessions, before, &after).await, Vec::<String>::new());

    let mixed: Option<uuid::Uuid> = sqlx::query_scalar(
        "select group_id from sessions where id = '00000000-0000-0000-0000-000000000002'",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(mixed, None);

    // Seeing a drone or controller: its team, or all flights.
    let equipment = "select serial as label, team_id, group_id from aircraft
                     union all select serial, team_id, group_id from remote_controllers";
    let before = format!("u.role in ('all_flights', 'admin') or {OWN_TEAM}");
    let after = holds(READS, "i.group_id");
    assert_eq!(differ(&db, equipment, &before, &after).await, Vec::<String>::new());

    // Flying for a team: its members.
    let teams = "select name as label, id as team_id from teams";
    let after = "exists (select 1 from group_members m
                         where m.user_id = u.id and m.group_id = i.team_id and m.role = 'pilot')";
    assert_eq!(differ(&db, teams, OWN_TEAM, after).await, Vec::<String>::new());
}
