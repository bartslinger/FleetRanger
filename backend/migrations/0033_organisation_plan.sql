-- An organisation's plan sets its limits, which are kept in the code.
-- Free: organisations made through sign-up. Unlimited: all others; the
-- operator changes a plan with the command-line tool.
alter table organisations
    add column plan text not null default 'unlimited'
        check (plan in ('free', 'unlimited'));
