-- What a paired controller last reported: when it last synced (a scan on a
-- connection its settings allow, also when it had no logs to upload) and the
-- app version it runs. Null until it first syncs.
alter table remote_controllers
    add column synced_at timestamptz,
    add column app_version text;
