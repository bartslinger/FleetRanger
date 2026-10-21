-- An area is one polygon: clipping that splits one makes separate areas with
-- the same name, so names needn't be unique (user's decision, 2026-10-21).
alter table areas drop constraint areas_name_key;
