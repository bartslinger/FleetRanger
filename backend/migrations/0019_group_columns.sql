-- The last two team columns become group columns; the code reads and writes
-- only the groups tables from here on. Every team became a group with the same
-- id (0018), so the values stay. The team_id columns on flights, aircraft and
-- remote_controllers are no longer used and go with teams.

-- The group chosen on the controller with the pilot; its flight gets it when
-- the log is processed. Only a record of the choice: a deleted group leaves
-- the log without one, as its flight has moved on.
alter table raw_logs rename column team_id to group_id;
alter table raw_logs
    drop constraint raw_logs_team_id_fkey,
    add constraint raw_logs_group_id_fkey
        foreign key (group_id) references groups (id) on delete set null;

-- The group the controller joins; null for the organisation. A code for a
-- deleted group goes with it, rather than pairing into the organisation.
alter table pairing_codes rename column team_id to group_id;
alter table pairing_codes
    drop constraint pairing_codes_team_id_fkey,
    add constraint pairing_codes_group_id_fkey
        foreign key (group_id) references groups (id) on delete cascade;
