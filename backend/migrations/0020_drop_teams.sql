-- Teams, the three fixed roles and holders are replaced by groups and roles
-- (migrations 0018 and 0019); see "Groups and roles" in docs/architecture.md.
-- Dropping a column also drops its foreign key and indexes (flights_team).
alter table flights drop column team_id;
alter table aircraft drop column team_id, drop column holder_user_id;
alter table remote_controllers drop column team_id, drop column holder_user_id;
alter table users drop column role;
drop table team_members;
drop table teams;
