-- Everything is in a group; the organisation only has Admin.

-- A group for what has none yet. A new install gets it as its only group.
insert into groups (name)
select 'General'
where not exists (select 1 from groups)
   or exists (select 1 from flights where group_id is null)
   or exists (select 1 from sessions where group_id is null)
   or exists (select 1 from aircraft where group_id is null)
   or exists (select 1 from remote_controllers where group_id is null)
on conflict (name) do nothing;

update flights set group_id = (select id from groups where name = 'General')
where group_id is null;
update sessions set group_id = (select id from groups where name = 'General')
where group_id is null;
update aircraft set group_id = (select id from groups where name = 'General')
where group_id is null;
update remote_controllers set group_id = (select id from groups where name = 'General')
where group_id is null;
-- Unused codes for the organisation go; they are created again for a group.
delete from pairing_codes where group_id is null;

alter table flights alter column group_id set not null;
alter table sessions alter column group_id set not null;
alter table aircraft alter column group_id set not null;
alter table remote_controllers alter column group_id set not null;
alter table pairing_codes alter column group_id set not null;

-- Roles on the whole organisation become the same roles in every group,
-- so nobody loses access.
insert into group_members (group_id, user_id, role)
select g.id, r.user_id, r.role
from organisation_roles r
cross join groups g
where r.role <> 'admin'
on conflict do nothing;

delete from organisation_roles where role <> 'admin';
alter table organisation_roles
    drop constraint organisation_roles_role_check,
    add constraint organisation_roles_role_check check (role = 'admin');

-- App roles on the organisation likewise move to every group.
insert into group_app_roles (group_id, user_id, app_role_id)
select g.id, r.user_id, r.app_role_id
from organisation_app_roles r
cross join groups g
on conflict do nothing;

drop table organisation_app_roles;
