-- Pairing with a code a user creates in the web app, which the pilot types into
-- the controller; see docs/pairing.md. It replaces controllers asking to pair
-- and a user approving them. The controller still generates its own token and
-- sends only its hash, which becomes `remote_controllers.token_sha256`.

drop table pairing_requests;

-- A code a user created for pairing one controller, until it's used,
-- revoked or expired.
create table pairing_codes (
    id          uuid primary key default uuidv7(),
    -- Typed on the controller; 8 characters without look-alikes.
    code        text not null unique,
    created_by  uuid not null references users (id) on delete cascade,
    -- The team the controller joins; null when the creator has none.
    team_id     uuid references teams (id),
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null,
    check (expires_at > created_at)
);
