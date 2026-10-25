-- Comments record what happened in a session; see "Comments" under
-- "Sessions" in docs/architecture.md. No cascade: code that deletes a
-- session deals with its comments first, so none get lost by mistake, and a
-- user who wrote comments is archived, not deleted.
create table session_comments (
    id          uuid primary key default uuidv7(),
    session_id  uuid not null references sessions (id),
    author_id   uuid not null references users (id),
    body        text not null check (btrim(body) <> '' and char_length(body) <= 4000),
    created_at  timestamptz not null default now(),
    edited_at   timestamptz check (edited_at >= created_at)
);

create index session_comments_session on session_comments (session_id, created_at);
