-- People who signed in with Google while sign-up was full. Only the owner
-- (System) uses it: no grant to fleetranger_app.
create table signup_waitlist (
    email      text primary key,      -- lower-case, verified by Google
    name       text not null,         -- as Google gives it
    created_at timestamptz not null default now()
);
alter table signup_waitlist enable row level security;
