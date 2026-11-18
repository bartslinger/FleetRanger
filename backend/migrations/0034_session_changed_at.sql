-- When a person last changed the session's flights or group; null while
-- only the automatic grouping has made or changed it. Only those sessions
-- (and without comments) are merged by themselves.
alter table sessions add column changed_at timestamptz;
