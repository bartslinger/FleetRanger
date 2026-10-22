-- Batteries, registered from the serials in imported logs like drones and
-- controllers, but with only a name to set; see "Batteries" in
-- docs/architecture.md.
create table batteries (
    id          uuid primary key default uuidv7(),
    serial      text not null unique,
    name        text,
    created_at  timestamptz not null default now()
);

-- A flight's batteries and what its log says about them. What a log doesn't
-- report is null; a battery's state now is that of its latest flight.
create table flight_batteries (
    flight_id           uuid not null references flights (id) on delete cascade,
    -- The battery's slot in the drone, from 0. On single-battery drones the
    -- log's battery records have index 0; on drones with more batteries DJI's
    -- own parser takes index 1 as slot 0, index 2 as slot 1, and so on.
    slot                smallint not null check (slot >= 0),
    battery_id          uuid not null references batteries (id),
    -- Charge at takeoff and at landing.
    charge_start_pct    smallint check (charge_start_pct between 0 and 100),
    charge_end_pct      smallint check (charge_end_pct between 0 and 100),
    -- What the battery holds now, and held new.
    full_capacity_mah   integer check (full_capacity_mah > 0),
    design_capacity_mah integer check (design_capacity_mah > 0),
    cycles              integer check (cycles >= 0),
    temperature_min_c   real,
    temperature_max_c   real,
    -- The lowest cell voltage, and the largest difference between cells.
    cell_voltage_min_v  real check (cell_voltage_min_v > 0),
    cell_spread_max_v   real check (cell_spread_max_v >= 0),
    primary key (flight_id, slot),
    unique (flight_id, battery_id)
);

create index flight_batteries_battery on flight_batteries (battery_id);

-- Read every log again for its battery. Logs that failed stay failed.
update raw_logs set status = 'pending', attempts = 0 where status = 'done';
