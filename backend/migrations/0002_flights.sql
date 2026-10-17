-- Flights derived from raw logs: at most one per log, and none for a log without a takeoff.
create table flights (
    id               uuid primary key default uuidv7(),
    raw_log_id       uuid not null unique references raw_logs (id),
    takeoff_at       timestamptz not null,
    landed_at        timestamptz not null,
    air_time_s       integer not null check (air_time_s >= 0),
    distance_m       double precision not null check (distance_m >= 0),
    aircraft_serial  text,
    rc_serial        text,
    -- GeoJSON LineString, one point per second: [longitude, latitude, height above takeoff].
    -- Only read for a single flight's page.
    track            jsonb not null,
    -- Display-only summaries (max altitude, camera and battery serials, ...), never time series.
    metrics          jsonb not null default '{}',
    check (landed_at >= takeoff_at)
);

create index flights_takeoff_at on flights (takeoff_at);
