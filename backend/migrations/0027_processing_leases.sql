-- Processing logs with one or several backends: a claim (lease) per log,
-- which expires if its backend dies, and when a log may be tried again.
alter table raw_logs
    add column next_attempt_at timestamptz not null default now(),
    add column claimed_until timestamptz;
drop index raw_logs_pending;
create index raw_logs_due on raw_logs (organisation_id, next_attempt_at)
    where status = 'pending';

-- DJI's key service as every backend sees it: the next free request slot
-- (one request per second, start to start) and an outage window.
create table dji_api (
    one             boolean primary key default true check (one),
    next_request_at timestamptz not null default now(),
    blocked_until   timestamptz,
    failures        integer not null default 0,
    last_error      text
);
insert into dji_api default values;
-- Install-wide, used only by the processing loop as the account that
-- created the tables: requests (fleetranger_app) get no access.
alter table dji_api enable row level security;
