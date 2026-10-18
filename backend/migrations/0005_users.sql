-- Everyone is a user: pilots and accounts become users, and login data moves
-- to logins. See "Identity and access" in docs/architecture.md.
--
-- A pilot keeps its id as a user, so flights, team memberships and equipment
-- holders stay valid. An account linked to a pilot becomes that user's login;
-- an account without a pilot becomes a user of its own, with its id.

create table users (
    id           uuid primary key default uuidv7(),
    name         text not null,
    role         text not null default 'member'
                 check (role in ('member', 'all_flights', 'admin')),
    created_at   timestamptz not null default now(),
    -- Can't log in and can't be chosen as a pilot.
    archived_at  timestamptz
);

create table logins (
    user_id       uuid primary key references users (id),
    email         text not null unique check (email = lower(email)),
    oidc_subject  text unique,
    created_at    timestamptz not null default now()
);

insert into users (id, name, role, created_at, archived_at)
select p.id, p.name, coalesce(a.role, 'member'), p.created_at, p.archived_at
from pilots p
left join accounts a on a.id = p.account_id;

insert into users (id, name, role, created_at, archived_at)
select a.id, coalesce(a.name, a.email), a.role, a.created_at, a.disabled_at
from accounts a
where not exists (select 1 from pilots p where p.account_id = a.id);

-- A disabled account linked to a pilot loses its login; the pilot stays.
insert into logins (user_id, email, oidc_subject, created_at)
select coalesce(p.id, a.id), a.email, a.oidc_subject, a.created_at
from accounts a
left join pilots p on p.account_id = a.id
where a.disabled_at is null or p.id is null;

-- Login sessions belong to the user.
alter table login_sessions add column user_id uuid references users (id);
update login_sessions s
set user_id = coalesce((select p.id from pilots p where p.account_id = s.account_id), s.account_id);
alter table login_sessions alter column user_id set not null;
alter table login_sessions drop column account_id;
create index login_sessions_user on login_sessions (user_id);

-- References to pilots now point at users, with the same ids.
alter table team_members drop constraint team_members_pilot_id_fkey;
alter table team_members rename column pilot_id to user_id;
alter table team_members add foreign key (user_id) references users (id);

alter table flights drop constraint flights_pilot_id_fkey;
alter table flights add foreign key (pilot_id) references users (id);

alter table aircraft drop constraint aircraft_holder_pilot_id_fkey;
alter table aircraft rename column holder_pilot_id to holder_user_id;
alter table aircraft add foreign key (holder_user_id) references users (id);

alter table remote_controllers drop constraint remote_controllers_holder_pilot_id_fkey;
alter table remote_controllers rename column holder_pilot_id to holder_user_id;
alter table remote_controllers add foreign key (holder_user_id) references users (id);

drop table pilots;
drop table accounts;
