-- People who can log in. An admin adds an account by email; the first login with
-- that verified email ties it to the provider's permanent subject ID.
create table accounts (
    id            uuid primary key default uuidv7(),
    -- How an admin adds someone; matched once on first login.
    email         text not null unique check (email = lower(email)),
    -- The provider's permanent ID for the person, set on first login.
    oidc_subject  text unique,
    -- From the provider, updated at each login; shown in the header.
    name          text,
    created_at    timestamptz not null default now(),
    -- Blocks login and revokes sessions, while keeping the account for the audit log.
    disabled_at   timestamptz
);

-- Server-side login sessions, named so to tell them apart from flight sessions.
create table login_sessions (
    id            uuid primary key default uuidv7(),
    account_id    uuid not null references accounts (id),
    -- The cookie holds the token; only its hash is stored.
    token_sha256  bytea not null unique check (length(token_sha256) = 32),
    created_at    timestamptz not null default now(),
    last_seen_at  timestamptz not null default now(),
    expires_at    timestamptz not null,
    revoked_at    timestamptz,
    ip            inet not null,
    user_agent    text
);

create index login_sessions_account on login_sessions (account_id);
