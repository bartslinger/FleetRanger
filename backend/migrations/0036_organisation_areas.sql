-- Areas are a feature an organisation turns on (user's decision, 2026-11-18):
-- off, nothing about them shows. Organisations that already have areas keep
-- seeing them.
alter table organisation_settings
    add column areas boolean not null default false;
update organisation_settings s
    set areas = true
    where exists (select 1 from areas a where a.organisation_id = s.organisation_id);
