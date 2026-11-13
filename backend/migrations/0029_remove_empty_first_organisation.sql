-- Migration 0026 made the data already there one organisation, named
-- "Organisation", also on a fresh install, where it holds nothing. Remove it
-- there: the first organisation (the oldest, as ids are UUIDv7), only while
-- that name is unchanged and it holds no data but its settings and the empty
-- group migration 0025 made. Installs whose data lives in it keep it.
do $$
declare
    first uuid;
    t record;
    used boolean;
begin
    select id into first from organisations where name = 'Organisation' order by id limit 1;
    if first is null or first <> (select id from organisations order by id limit 1) then
        return;
    end if;
    for t in
        select table_name from information_schema.columns
        where table_schema = 'public' and column_name = 'organisation_id'
          and table_name not in ('organisation_settings', 'groups')
    loop
        execute format('select exists (select 1 from %I where organisation_id = $1)', t.table_name)
            into used using first;
        if used then
            return;
        end if;
    end loop;
    delete from groups where organisation_id = first;
    delete from organisation_settings where organisation_id = first;
    delete from organisations where id = first;
end $$;
