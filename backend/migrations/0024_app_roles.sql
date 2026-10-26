-- Roles an app declares, typed in by an admin on the app's page under
-- Apps; see "Apps" in docs/architecture.md. The app decides what a role
-- allows; FleetRanger only keeps who holds it, on a group or on the
-- organisation. Removing an app removes its roles and everyone's grants
-- of them.
create table app_roles (
    id         uuid primary key default uuidv7(),
    client_id  uuid not null references oauth_clients (id) on delete cascade,
    -- What the app sees in /api/v1/me.
    key        text not null check (key ~ '^[a-z][a-z0-9_]{0,63}$'),
    -- What admins see in the roles dialogs.
    name       text not null check (btrim(name) <> '' and char_length(name) <= 100),
    unique (client_id, key),
    unique (client_id, name)
);

-- App roles in a group. Like group_members, holding one makes the person
-- a member of the group.
create table group_app_roles (
    group_id     uuid not null references groups (id) on delete cascade,
    user_id      uuid not null references users (id) on delete cascade,
    app_role_id  uuid not null references app_roles (id) on delete cascade,
    primary key (group_id, user_id, app_role_id)
);

create index group_app_roles_user on group_app_roles (user_id);

-- App roles on the whole organisation.
create table organisation_app_roles (
    user_id      uuid not null references users (id) on delete cascade,
    app_role_id  uuid not null references app_roles (id) on delete cascade,
    primary key (user_id, app_role_id)
);
