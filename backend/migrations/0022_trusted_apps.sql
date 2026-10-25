-- Apps an admin added on the Apps page: trusted, so they skip the consent
-- screen. Self-registered apps (assistants) aren't. Removing a trusted app
-- deletes its users' codes and tokens through the cascades on oauth_codes
-- and oauth_grants.
alter table oauth_clients
    add column trusted boolean not null default false;
