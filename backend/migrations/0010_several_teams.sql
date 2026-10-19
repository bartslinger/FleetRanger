-- A user can fly for several teams, e.g. a pilot borrowing another team's
-- drones (user's decision, 2026-10-19), but is in each team once.
alter table team_members drop constraint team_members_pkey;
alter table team_members add primary key (user_id, team_id);

-- The team chosen on the controller with the pilot, when the log was written:
-- its flight belongs to it. Null: none chosen, or a log that didn't come from
-- the controller app (or from a version without the choice).
alter table raw_logs add column team_id uuid references teams (id);
