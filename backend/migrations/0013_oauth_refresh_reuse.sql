-- The refresh token a grant had before its last refresh. Presented again,
-- it was copied: the grant ends, cutting off whoever holds either token
-- (refresh token reuse detection, RFC 9700; user's decision, 2026-10-20).
alter table oauth_grants
    add column previous_refresh_sha256 bytea unique
        check (length(previous_refresh_sha256) = 32);

-- For removing apps that were never connected.
create index oauth_grants_client on oauth_grants (client_id);
