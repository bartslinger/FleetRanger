-- Sessions group consecutive flights for reporting; see "Sessions and
-- operations" in docs/architecture.md. A session's pilots, times and totals
-- come from its flights, so it stores nothing else. Empty sessions are deleted.
create table sessions (
    id          uuid primary key default uuidv7(),
    created_at  timestamptz not null default now()
);

alter table flights add column session_id uuid references sessions (id);
create index flights_session on flights (session_id);
