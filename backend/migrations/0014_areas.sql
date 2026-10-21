-- Named parts of the land that flights are reported by; see "Areas" in
-- docs/architecture.md. They never overlap: an area is clipped against the
-- others when it's saved.
create table areas (
    id          uuid primary key default uuidv7(),
    name        text not null unique,
    -- GeoJSON Polygon or MultiPolygon, [longitude, latitude].
    boundary    jsonb not null,
    created_at  timestamptz not null default now()
);

-- A flight's time and distance per area, computed from its track: a row for
-- every area with at least one of its track points, also when its share
-- rounds to 0 seconds. The rows add up to the flight's totals with the part
-- outside all areas, which isn't stored. Recomputed when areas change.
create table flight_areas (
    flight_id   uuid not null references flights (id) on delete cascade,
    area_id     uuid not null references areas (id) on delete cascade,
    air_time_s  integer not null check (air_time_s >= 0),
    distance_m  double precision not null check (distance_m >= 0),
    primary key (flight_id, area_id)
);

create index flight_areas_area on flight_areas (area_id);
