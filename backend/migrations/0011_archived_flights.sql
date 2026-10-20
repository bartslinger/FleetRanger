-- An archived flight is kept but left out everywhere, as if it hadn't
-- happened, until it's restored (user's decision, 2026-10-20). It is never
-- in a session. Who archived it is for the audit log.
alter table flights add column archived_at timestamptz;
