-- Pairing links a remote controller to the install; see "Clients" in
-- docs/architecture.md. The controller generates its own token and sends only
-- its hash; when a user approves the request, that hash becomes the
-- controller's token. The token itself never leaves the controller.

-- A controller asking to pair, until a user approves it or it expires.
create table pairing_requests (
    -- Shown on the controller and in its QR code.
    code          text primary key,
    token_sha256  bytea not null unique,
    serial        text not null,
    model         text not null,
    expires_at    timestamptz not null
);

-- A paired controller's token, as a hash; null when it isn't paired.
alter table remote_controllers add column token_sha256 bytea unique;
