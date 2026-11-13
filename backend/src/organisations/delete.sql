-- Deletes an organisation ($1) with every row of its data: one statement, so
-- the foreign keys are checked only once all of it is gone, whatever the
-- order. Every table with an organisation_id is here (a test checks).
-- Answers how many flights went.
with
  flight_areas as (delete from flight_areas where organisation_id = $1),
  flight_batteries as (delete from flight_batteries where organisation_id = $1),
  session_comments as (delete from session_comments where organisation_id = $1),
  dji_key_requests as (delete from dji_key_requests where organisation_id = $1),
  flights as (delete from flights where organisation_id = $1 returning 1),
  sessions as (delete from sessions where organisation_id = $1),
  raw_logs as (delete from raw_logs where organisation_id = $1),
  areas as (delete from areas where organisation_id = $1),
  batteries as (delete from batteries where organisation_id = $1),
  aircraft as (delete from aircraft where organisation_id = $1),
  remote_controllers as (delete from remote_controllers where organisation_id = $1),
  pairing_codes as (delete from pairing_codes where organisation_id = $1),
  group_app_roles as (delete from group_app_roles where organisation_id = $1),
  group_members as (delete from group_members where organisation_id = $1),
  app_roles as (delete from app_roles where organisation_id = $1),
  oauth_codes as (delete from oauth_codes where organisation_id = $1),
  oauth_grants as (delete from oauth_grants where organisation_id = $1),
  login_sessions as (delete from login_sessions where organisation_id = $1),
  logins as (delete from logins where organisation_id = $1),
  organisation_roles as (delete from organisation_roles where organisation_id = $1),
  groups as (delete from groups where organisation_id = $1),
  oauth_clients as (delete from oauth_clients where organisation_id = $1),
  users as (delete from users where organisation_id = $1),
  organisation_settings as (delete from organisation_settings where organisation_id = $1),
  organisation as (delete from organisations where id = $1)
select count(*) as "flights!" from flights
