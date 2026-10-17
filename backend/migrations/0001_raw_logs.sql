-- Raw DJI flight logs: the files on disk and their processing state.
create table raw_logs (
    id            uuid primary key default uuidv7(),
    sha256        bytea not null unique check (length(sha256) = 32),
    -- Relative to the flight log directory.
    path          text not null unique,
    size_bytes    bigint not null,
    received_at   timestamptz not null default now(),
    status        text not null default 'pending'
                  check (status in ('pending', 'done', 'failed')),
    attempts      integer not null default 0,
    last_error    text,
    processed_at  timestamptz
);

create index raw_logs_pending on raw_logs (received_at) where status = 'pending';

-- Every request for decryption keys sent to DJI, exactly as sent and received.
create table dji_key_requests (
    id              uuid primary key default uuidv7(),
    raw_log_id      uuid not null references raw_logs (id),
    requested_at    timestamptz not null default now(),
    -- The API key is sent as a header and is not stored.
    request_body    text not null,
    -- Null if DJI never answered.
    response_status integer,
    response_body   text,
    error           text
);

create index dji_key_requests_raw_log on dji_key_requests (raw_log_id);
