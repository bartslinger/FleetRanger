-- OAuth: apps such as AI assistants act for a user with tokens FleetRanger
-- issues, after the user allowed it (user's decisions, 2026-10-20). Tokens
-- and codes are stored only as SHA-256 hashes. See "Identity and access" in
-- docs/architecture.md.

-- Apps that registered themselves (OAuth dynamic client registration), which
-- grants nothing by itself. Its id is the client_id.
create table oauth_clients (
    id             uuid primary key default uuidv7(),
    -- As the app gave it; shown on the consent page.
    name           text not null,
    redirect_uris  text[] not null,
    created_at     timestamptz not null default now()
);

-- One-time codes between the user's "Allow" and the app's token request.
create table oauth_codes (
    code_sha256     bytea primary key check (length(code_sha256) = 32),
    client_id       uuid not null references oauth_clients (id) on delete cascade,
    user_id         uuid not null references users (id) on delete cascade,
    redirect_uri    text not null,
    -- PKCE: the S256 challenge the app's verifier must match.
    code_challenge  text not null,
    -- What the user allowed (e.g. flights:read), and for which part
    -- (the resource, e.g. https://fleet.example.org/mcp).
    scopes          text[] not null,
    resource        text not null,
    expires_at      timestamptz not null
);

-- An app a user connected: listed in the web app, where the user can
-- disconnect it. Its refresh token is replaced at every refresh.
create table oauth_grants (
    id                    uuid primary key default uuidv7(),
    client_id             uuid not null references oauth_clients (id) on delete cascade,
    user_id               uuid not null references users (id) on delete cascade,
    scopes                text[] not null,
    resource              text not null,
    access_token_sha256   bytea not null unique check (length(access_token_sha256) = 32),
    access_expires_at     timestamptz not null,
    refresh_token_sha256  bytea not null unique check (length(refresh_token_sha256) = 32),
    created_at            timestamptz not null default now(),
    last_used_at          timestamptz not null default now()
);

create index oauth_grants_user on oauth_grants (user_id);
