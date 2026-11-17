-- Who invited the person a login is for: their name shows with the invitation.
-- An invitation stays when the user who sent it is deleted.
alter table logins
    add column invited_by uuid,
    add foreign key (organisation_id, invited_by)
        references users (organisation_id, id)
        on delete set null (invited_by);
