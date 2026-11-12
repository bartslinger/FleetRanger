-- Several organisations per install: an organisation on every row of an
-- organisation's data, kept apart by row-level security.

-- 1. Organisations, kept by the operator, and their settings, which were
-- environment variables and which an organisation may later change itself.
create table organisations (
    id          uuid primary key default uuidv7(),
    -- Not unique: unrelated organisations may share a name, and a taken name
    -- would tell others it exists. The command-line tool works with ids.
    name        text not null,
    created_at  timestamptz not null default now(),
    archived_at timestamptz
);
create table organisation_settings (
    organisation_id     uuid primary key default current_setting('app.organisation_id')::uuid
                        references organisations (id),
    time_zone           text not null default 'UTC',
    session_gap_minutes integer not null default 120 check (session_gap_minutes > 0),
    mcp                 boolean not null default false
);
-- The existing data becomes one organisation; the deploy sets its name and
-- settings with the command-line tool.
insert into organisations (name) values ('Organisation');
insert into organisation_settings (organisation_id) select id from organisations;

-- 2. An organisation on every table of an organisation's data. New rows take
-- the request's organisation by themselves.
do $$
declare t text;
begin
    foreach t in array array[
        'users', 'groups', 'group_members', 'organisation_roles',
        'aircraft', 'remote_controllers', 'batteries', 'areas', 'pairing_codes',
        'raw_logs', 'dji_key_requests', 'flights', 'flight_areas', 'flight_batteries',
        'sessions', 'session_comments', 'app_roles', 'group_app_roles',
        'oauth_codes', 'oauth_grants', 'logins', 'login_sessions'
    ] loop
        execute format('alter table %I add column organisation_id uuid references organisations (id)', t);
        execute format('update %I set organisation_id = (select id from organisations)', t);
        execute format('alter table %I alter column organisation_id set not null', t);
        execute format($f$alter table %I alter column organisation_id
                          set default current_setting('app.organisation_id')::uuid$f$, t);
    end loop;
end $$;

-- 3. Logins and login sessions belong to their user's organisation, like
-- everything else (part 2): someone in two organisations has a login in each,
-- and the same Google account is what links them. Their emails and Google
-- accounts become unique per organisation (part 5).

-- 4. An app an admin adds (a trusted app) belongs to that organisation; an
-- app that registers itself (an assistant) belongs to none, and each user's
-- permission for it (oauth_grants) is per organisation. Trusted now means
-- belonging to an organisation.
alter table oauth_clients add column organisation_id uuid references organisations (id);
update oauth_clients set organisation_id = (select id from organisations) where trusted;
alter table oauth_clients drop column trusted;
alter table oauth_clients add unique (organisation_id, id);
alter table app_roles
    drop constraint app_roles_client_id_fkey,
    add foreign key (organisation_id, client_id) references oauth_clients (organisation_id, id)
        on delete cascade;

-- 5. Uniqueness per organisation: the same drone, controller, battery, log or
-- group name, and the same email or Google account, can exist in two
-- organisations. Pairing codes and tokens stay unique on the install, as they
-- are looked up before an organisation is known.
alter table aircraft drop constraint aircraft_serial_key, add unique (organisation_id, serial);
alter table remote_controllers drop constraint remote_controllers_serial_key,
    add unique (organisation_id, serial);
alter table batteries drop constraint batteries_serial_key, add unique (organisation_id, serial);
alter table raw_logs drop constraint raw_logs_sha256_key, add unique (organisation_id, sha256),
    drop constraint raw_logs_path_key, add unique (organisation_id, path);
alter table groups drop constraint groups_name_key, add unique (organisation_id, name);
alter table logins drop constraint logins_email_key, add unique (organisation_id, email),
    drop constraint logins_oidc_subject_key, add unique (organisation_id, oidc_subject);
-- App roles stay unique per app: an app with roles is in one organisation.

-- 6. References stay within an organisation: a flight's pilot, group, session
-- and log are the flight's organisation's.
alter table users add unique (organisation_id, id);
alter table groups add unique (organisation_id, id);
alter table raw_logs add unique (organisation_id, id);
alter table flights add unique (organisation_id, id);
alter table sessions add unique (organisation_id, id);
alter table areas add unique (organisation_id, id);
alter table batteries add unique (organisation_id, id);
alter table app_roles add unique (organisation_id, id);

alter table raw_logs
    drop constraint raw_logs_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id)
        on delete set null (group_id),
    drop constraint raw_logs_pilot_id_fkey,
    add foreign key (organisation_id, pilot_id) references users (organisation_id, id);
alter table dji_key_requests
    drop constraint dji_key_requests_raw_log_id_fkey,
    add foreign key (organisation_id, raw_log_id) references raw_logs (organisation_id, id);
alter table flights
    drop constraint flights_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id),
    drop constraint flights_pilot_id_fkey,
    add foreign key (organisation_id, pilot_id) references users (organisation_id, id),
    drop constraint flights_raw_log_id_fkey,
    add foreign key (organisation_id, raw_log_id) references raw_logs (organisation_id, id),
    drop constraint flights_session_id_fkey,
    add foreign key (organisation_id, session_id) references sessions (organisation_id, id);
alter table aircraft
    drop constraint aircraft_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id);
alter table remote_controllers
    drop constraint remote_controllers_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id);
alter table sessions
    drop constraint sessions_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id);
alter table session_comments
    drop constraint session_comments_author_id_fkey,
    add foreign key (organisation_id, author_id) references users (organisation_id, id),
    drop constraint session_comments_session_id_fkey,
    add foreign key (organisation_id, session_id) references sessions (organisation_id, id);
alter table flight_areas
    drop constraint flight_areas_flight_id_fkey,
    add foreign key (organisation_id, flight_id) references flights (organisation_id, id)
        on delete cascade,
    drop constraint flight_areas_area_id_fkey,
    add foreign key (organisation_id, area_id) references areas (organisation_id, id)
        on delete cascade;
alter table flight_batteries
    drop constraint flight_batteries_flight_id_fkey,
    add foreign key (organisation_id, flight_id) references flights (organisation_id, id)
        on delete cascade,
    drop constraint flight_batteries_battery_id_fkey,
    add foreign key (organisation_id, battery_id) references batteries (organisation_id, id);
alter table pairing_codes
    drop constraint pairing_codes_created_by_fkey,
    add foreign key (organisation_id, created_by) references users (organisation_id, id)
        on delete cascade,
    drop constraint pairing_codes_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id)
        on delete cascade;
alter table group_members
    drop constraint group_members_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id)
        on delete cascade,
    drop constraint group_members_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id)
        on delete cascade;
alter table organisation_roles
    drop constraint organisation_roles_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id)
        on delete cascade;
alter table group_app_roles
    drop constraint group_app_roles_group_id_fkey,
    add foreign key (organisation_id, group_id) references groups (organisation_id, id)
        on delete cascade,
    drop constraint group_app_roles_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id)
        on delete cascade,
    drop constraint group_app_roles_app_role_id_fkey,
    add foreign key (organisation_id, app_role_id) references app_roles (organisation_id, id)
        on delete cascade;
alter table oauth_codes
    drop constraint oauth_codes_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id)
        on delete cascade,
    drop constraint oauth_codes_impersonated_by_fkey,
    add foreign key (organisation_id, impersonated_by) references users (organisation_id, id)
        on delete cascade;
alter table oauth_grants
    drop constraint oauth_grants_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id)
        on delete cascade,
    drop constraint oauth_grants_impersonated_by_fkey,
    add foreign key (organisation_id, impersonated_by) references users (organisation_id, id)
        on delete cascade;
alter table logins
    drop constraint logins_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id);
alter table login_sessions
    drop constraint login_sessions_user_id_fkey,
    add foreign key (organisation_id, user_id) references users (organisation_id, id);

-- 7. Row-level security. Normal requests use the database account
-- fleetranger_app, which sees only the rows of the organisation set in its
-- transaction (app.organisation_id), and nothing at all without one. The
-- account fleetranger, which created the tables, isn't restricted: the
-- backend uses it only to apply database changes at startup and for the few
-- jobs that look across organisations, such as finding which organisation a
-- token belongs to or taking the next log to process. The deploy gives
-- fleetranger_app its password; accounts belong to the whole PostgreSQL
-- server, which the tests' databases share, so it may exist already.
do $$
begin
    create role fleetranger_app nologin;
exception when duplicate_object then null;
end $$;
do $$
declare t text;
begin
    foreach t in array array[
        'users', 'groups', 'group_members', 'organisation_roles',
        'aircraft', 'remote_controllers', 'batteries', 'areas', 'pairing_codes',
        'raw_logs', 'dji_key_requests', 'flights', 'flight_areas', 'flight_batteries',
        'sessions', 'session_comments', 'app_roles', 'group_app_roles',
        'oauth_codes', 'oauth_grants', 'logins', 'login_sessions'
    ] loop
        execute format('alter table %I enable row level security', t);
        execute format($f$create policy organisation on %I to fleetranger_app
                          using (organisation_id = current_setting('app.organisation_id')::uuid)$f$, t);
        execute format('grant select, insert, update, delete on %I to fleetranger_app', t);
    end loop;
end $$;
-- Apps: an organisation's own, and those that registered themselves.
alter table oauth_clients enable row level security;
create policy organisation on oauth_clients to fleetranger_app
    using (organisation_id is null
           or organisation_id = current_setting('app.organisation_id')::uuid)
    with check (organisation_id = current_setting('app.organisation_id')::uuid);
grant select, insert, update, delete on oauth_clients to fleetranger_app;
-- Settings: the organisation's own row, read and changed but not created or
-- removed, which is the operator's work.
alter table organisation_settings enable row level security;
create policy organisation on organisation_settings to fleetranger_app
    using (organisation_id = current_setting('app.organisation_id')::uuid);
grant select, update on organisation_settings to fleetranger_app;
-- The organisation itself, only to read.
alter table organisations enable row level security;
create policy organisation on organisations to fleetranger_app
    using (id = current_setting('app.organisation_id')::uuid);
grant select on organisations to fleetranger_app;
