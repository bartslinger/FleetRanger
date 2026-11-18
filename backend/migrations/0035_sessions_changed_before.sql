-- Sessions from before 0034: whether people changed them isn't known, so all
-- count as changed and are never merged by themselves. The time is the
-- latest that is known: its last upload or comment.
update sessions s set changed_at = coalesce(greatest(
    (select max(l.received_at) from flights f
     join raw_logs l on l.id = f.raw_log_id where f.session_id = s.id),
    (select max(coalesce(c.edited_at, c.created_at)) from session_comments c
     where c.session_id = s.id)
), now()) where changed_at is null;
