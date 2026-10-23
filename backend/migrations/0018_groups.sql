-- Groups and roles, added beside teams; see "Groups and roles" in
-- docs/architecture.md. The code keeps using teams, users.role and holders
-- until they are dropped. The fill below gives everyone the access they
-- have now.

-- A group holds flights, sessions, drones and controllers. Groups aren't
-- archived; only an empty one can be deleted.
create table groups (
    id          uuid primary key default uuidv7(),
    name        text not null unique,
    created_at  timestamptz not null default now()
);

-- A membership and the person's roles in the group, one row per role, so
-- every membership has at least one role. Admin is organisation-wide only.
create table group_members (
    group_id  uuid not null references groups (id) on delete cascade,
    user_id   uuid not null references users (id) on delete cascade,
    role      text not null check (role in ('viewer', 'pilot', 'editor')),
    primary key (group_id, user_id, role)
);

create index group_members_user on group_members (user_id);

-- Roles on the whole organisation.
create table organisation_roles (
    user_id  uuid not null references users (id) on delete cascade,
    role     text not null check (role in ('viewer', 'pilot', 'editor', 'admin')),
    primary key (user_id, role)
);

-- The group each item is in; null means directly in the organisation
-- (for flights, the queue).
alter table flights add column group_id uuid references groups (id);
create index flights_group on flights (group_id, takeoff_at);

alter table sessions add column group_id uuid references groups (id);
alter table aircraft add column group_id uuid references groups (id);
alter table remote_controllers add column group_id uuid references groups (id);

-- Teams become groups with the same ids, so every team_id is its group's id.
insert into groups (id, name, created_at)
select id, name, created_at from teams;

-- Every team member becomes a member with Pilot and Editor.
insert into group_members (group_id, user_id, role)
select m.team_id, m.user_id, r.role
from team_members m
cross join (values ('pilot'), ('editor')) as r (role);

-- Admins get Admin and the all-flights role Viewer on the whole organisation.
insert into organisation_roles (user_id, role)
select id, case role when 'admin' then 'admin' else 'viewer' end
from users
where role in ('admin', 'all_flights');

update flights set group_id = team_id where team_id is not null;
update aircraft set group_id = team_id where team_id is not null;
update remote_controllers set group_id = team_id where team_id is not null;

-- A session whose flights all have one team gets that group. Any other
-- session (teams mixed, or flights in the queue) stays in the organisation.
update sessions s
set group_id = one.team_id
from (
    select session_id, (array_agg(team_id))[1] as team_id
    from flights
    where session_id is not null
    group by session_id
    having count(distinct team_id) = 1 and bool_and(team_id is not null)
) one
where s.id = one.session_id;
