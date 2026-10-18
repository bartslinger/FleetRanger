-- Teams, pilots, account roles, and the organisation's drones and remote
-- controllers. See "Identity and access" in docs/architecture.md.

-- Archived rather than deleted, so flights and history keep their team.
create table teams (
    id           uuid primary key default uuidv7(),
    name         text not null unique,
    created_at   timestamptz not null default now(),
    archived_at  timestamptz
);

-- A pilot is separate from a login account. The remote controller app may
-- supply the id, as it can create pilots offline.
create table pilots (
    id           uuid primary key default uuidv7(),
    name         text not null,
    account_id   uuid unique references accounts (id),
    created_at   timestamptz not null default now(),
    archived_at  timestamptz
);

-- One team per pilot for now (pilot_id is the key); a separate table so
-- several teams per pilot remain possible later.
create table team_members (
    pilot_id  uuid primary key references pilots (id),
    team_id   uuid not null references teams (id)
);

create index team_members_team on team_members (team_id);

alter table accounts add column role text not null default 'member'
    check (role in ('member', 'all_flights', 'admin'));

-- Drones and remote controllers are registered from the serials in logs. They
-- are separate tables because they will differ: controllers get enrollment and
-- sync settings. The holder is informational and grants no access.
create table aircraft (
    id               uuid primary key default uuidv7(),
    serial           text not null unique,
    name             text,
    team_id          uuid references teams (id),
    holder_pilot_id  uuid references pilots (id),
    created_at       timestamptz not null default now()
);

create table remote_controllers (
    id               uuid primary key default uuidv7(),
    serial           text not null unique,
    name             text,
    team_id          uuid references teams (id),
    holder_pilot_id  uuid references pilots (id),
    created_at       timestamptz not null default now()
);

-- A flight without a team is in the organisation's queue.
alter table flights
    add column team_id   uuid references teams (id),
    add column pilot_id  uuid references pilots (id);

create index flights_team on flights (team_id, takeoff_at);
create index flights_pilot on flights (pilot_id, takeoff_at);
