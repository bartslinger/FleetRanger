-- The pilot chosen on the controller when the log was written: its flight gets
-- this pilot when it's processed. Null: "Not in the list", nobody chosen, or a
-- log that didn't come from the controller app.
alter table raw_logs add column pilot_id uuid references users (id);
