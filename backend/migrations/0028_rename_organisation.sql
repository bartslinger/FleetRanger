-- Admins rename their own organisation. The policy from migration 0026
-- already limits fleetranger_app to its own row; creating and removing
-- organisations stays with the operator.
grant update (name) on organisations to fleetranger_app;
