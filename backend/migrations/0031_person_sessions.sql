-- A login session is the person's: the account they logged in with, then,
-- once chosen, their user in an organisation.
alter table login_sessions
    add column subject text,
    add column email text check (email = lower(email)),
    add column name text;

update login_sessions s
set subject = l.oidc_subject, email = l.email, name = u.name
from logins l, users u
where l.user_id = s.user_id and u.id = s.user_id;

-- Sessions whose login has gone can't be used anyway.
delete from login_sessions where subject is null;

-- A session in an organisation is for a login there, with the same
-- account: the database keeps them from drifting apart.
alter table logins
    add unique (organisation_id, user_id, oidc_subject);

alter table login_sessions
    alter column subject set not null,
    alter column user_id drop not null,
    alter column organisation_id drop not null,
    add check ((user_id is null) = (organisation_id is null)),
    drop constraint login_sessions_organisation_id_user_id_fkey,
    add foreign key (organisation_id, user_id, subject)
        references logins (organisation_id, user_id, oidc_subject);
