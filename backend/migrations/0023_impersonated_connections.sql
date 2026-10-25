-- An admin impersonating someone who opens an app connects it as that person
-- (see "Impersonation in apps" in docs/architecture.md). Such a connection
-- records the admin, ends with the impersonation and isn't in the person's
-- connected apps.
alter table oauth_codes
    add column impersonated_by uuid references users (id) on delete cascade;
alter table oauth_grants
    add column impersonated_by uuid references users (id) on delete cascade;
